#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, BankMsg, coins, StdError, Uint128, Uint64};
use cw2::set_contract_version;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, PostCountResponse, AuthMsg, PostMsg, LatestPostsResponse, GetBalanceResponse};
use cw_utils::must_pay;
use cw_auth::authorize;
use crate::state::{Post, State, STATE, FUNDS, POSTS_COUNT, POSTS};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-posts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const COIN_DENOM: &str = "ujunox";
const DEFAULT_POST_LIMIT: u8 = 10;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        name_chars: msg.name_chars,
        post_chars: msg.post_chars,
        agent_cut: msg.agent_cut,
        post_fee: msg.post_fee,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    POSTS_COUNT.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Post(msg) => post(deps, authorize(msg, &info, &env)?),
        ExecuteMsg::DepositFunds { amount } => deposit_funds(deps, info, amount),
        ExecuteMsg::WithdrawFunds { amount } => withdraw_funds(deps, info, amount)
    }
}

pub fn post(
    mut deps: DepsMut,
    msg: AuthMsg<PostMsg>
) -> Result<Response, ContractError> {
    let user_addr = msg.auth_token.user;
    let username = msg.auth_token.meta.username;
    let name_length = username.len() as u8;
    let content = msg.message.content;
    let content_length = content.len() as u8;

    let state = STATE.load(deps.storage)?;

    if content_length > state.post_chars {
        return Err(ContractError::ExceededCharLimit { field: "content".to_string(), length: content_length, max: state.post_chars });
    } else if name_length > state.name_chars {
        return Err(ContractError::ExceededCharLimit { field: "username".to_string(), length: name_length, max: state.name_chars });
    }

    let agent_cut: u64 = state.agent_cut.into();
    move_funds(&mut deps, &user_addr, &state.owner, state.post_fee * Decimal::percent(100 - agent_cut))?;
    move_funds(&mut deps, &user_addr, &msg.auth_token.agent, state.post_fee * Decimal::percent(agent_cut))?;

    let id = POSTS_COUNT.update(deps.storage, |count| -> Result<_, ContractError> {
        Ok(count + 1)
    })?;

    POSTS.save(deps.storage, id, &Post {
        user_addr,
        username,
        content,
    })?;

    Ok(Response::new().add_attribute("method", "post"))
}

fn move_funds(
    deps: &mut DepsMut,
    from_addr: &Addr,
    to_addr: &Addr,
    amount: Uint128,
) -> Result<bool, ContractError> {
    take_funds(deps, from_addr, amount)?;
    give_funds(deps, to_addr, amount)?;
    Ok(true)
}

fn take_funds(
    deps: &mut DepsMut,
    from_addr: &Addr,
    amount: Uint128,
) -> Result<Uint128, ContractError> {
    Ok(Uint128::from(FUNDS.update(deps.storage, from_addr, |funds| -> Result<_, ContractError> {
        let amount = amount.u128();
        let funds = funds.unwrap_or(0);
        if funds < amount {
            return Err(ContractError::InsufficientFunds { got: funds, needed: amount });
        }
        Ok(funds - amount)
    })?))
}

fn give_funds(
    deps: &mut DepsMut,
    to_addr: &Addr,
    amount: Uint128,
) -> StdResult<Uint128> {
    Ok(Uint128::from(FUNDS.update(deps.storage, to_addr, |funds| -> Result<_, StdError> {
        Ok(funds.unwrap_or(0) + amount.u128())
    })?))
}

pub fn deposit_funds(
    mut deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let payment = must_pay(&info, COIN_DENOM)?;
    if payment < amount {
        return Err(ContractError::InsufficientFunds { got: payment.u128(), needed: amount.u128() });
    }

    let balance = give_funds(&mut deps, &info.sender, amount)?;

    Ok(Response::new()
        .add_attribute("action", "deposit_funds")
        .add_attribute("balance", balance.to_string()))
}

pub fn withdraw_funds(
    mut deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {

    let balance = take_funds(&mut deps, &info.sender, amount)?;

    Ok(Response::new()
        .add_attribute("method", "withdraw_funds")
        .add_attribute("balance", balance.to_string())
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(amount.u128(), COIN_DENOM),
        }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PostCount {} => to_binary(&post_count(deps)?),
        QueryMsg::LatestPosts { limit } => to_binary(&latest_posts(deps, limit)?),
        QueryMsg::GetBalance { addr } => to_binary(&get_balance(deps, addr)?),
    }
}

fn post_count(deps: Deps) -> StdResult<PostCountResponse> {
    let count = Uint64::from(POSTS_COUNT.load(deps.storage)?);
    Ok(PostCountResponse { count })
}

fn latest_posts(deps: Deps, limit: Option<u8>) -> StdResult<LatestPostsResponse> {
    let limit = limit.unwrap_or(DEFAULT_POST_LIMIT);

    let posts: StdResult<Vec<Post>> = POSTS.range(deps.storage, None, None, Order::Descending)
            .take(limit as usize)
            .map(|item| item.map(|(_, post)| post))
            .collect();
    Ok(LatestPostsResponse { posts: posts? })
}

fn get_balance(deps: Deps, addr: Addr) -> StdResult<GetBalanceResponse> {
    let balance = Uint128::from(FUNDS.load(deps.storage, &addr).unwrap_or(0));
    Ok(GetBalanceResponse { balance })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { name_chars: 20, post_chars: 140, post_fee: Uint128::from(10000u128), agent_cut: 90 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::PostCount {}).unwrap();
        let value: PostCountResponse = from_binary(&res).unwrap();
        assert_eq!(0, value.count.u64());
    }
}
