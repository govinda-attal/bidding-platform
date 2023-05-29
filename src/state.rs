use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct CommissionParams {
    pub part: Decimal,
    pub minimum_tokens: Uint128,
}

pub const ITEM: Item<String> = Item::new("item");
pub const BID_DENOM: Item<String> = Item::new("bid_denom");
pub const OWNER: Item<Addr> = Item::new("owner");
pub const COMMISSION_PARAMS: Item<CommissionParams> = Item::new("commission");
pub const BID_OPEN: Item<bool> = Item::new("bid_open");
pub const BIDS: Map<Addr, Uint128> = Map::new("bids");
pub const HIGHEST_BID: Item<Uint128> = Item::new("highest_bid");
pub const HIGHEST_BIDDER: Item<Addr> = Item::new("highest_bidder");
