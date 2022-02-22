use cosmwasm_std::{Addr, Uint128, Uint64};
use cw_auth::{Authorized, MsgWithAuth};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::Post;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AuthMeta {
    pub username: String,
}

pub type AuthMsg<T> = Authorized<AuthMeta, T>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub char_limit: u8,
    pub post_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PostMsg {
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Post(MsgWithAuth<PostMsg>),
    DepositFunds { amount: Uint128 },
    WithdrawFunds { amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    PostCount {},
    LatestPosts { limit: Option<u8> },
    GetBalance { addr: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PostCountResponse {
    pub count: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LatestPostsResponse {
    pub posts: Vec<Post>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetBalanceResponse {
    pub balance: Uint128,
}
