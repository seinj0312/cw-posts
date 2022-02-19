use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub char_limit: u8,
    pub post_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Post {
    pub poster: Addr,
    pub content: String,
}

pub const STATE: Item<State> = Item::new("state");
pub const FUNDS: Map<&Addr, u128> = Map::new("funds");
pub const POSTS: Map<u64, Post> = Map::new("posts");
pub const POSTS_COUNT: Item<u64> = Item::new("posts_count");
