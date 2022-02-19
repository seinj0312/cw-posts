use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use cw_auth::AuthError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AuthError(#[from] AuthError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Insufficent funds (got {got:?}, needed {needed:?})")]
    InsufficientFunds { got: u128, needed: u128 },

    #[error("Post too long (length {length:?}, max {max:?})")]
    PostTooLong { length: u8, max: u8 },
}
