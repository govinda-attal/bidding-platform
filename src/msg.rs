use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub item: String,
    pub bid_denom: String,
    pub owner: Option<String>,
    pub commission_minimum_tokens: Uint128,
    pub commission_part: Decimal,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(HighestBidResponse)]
    HighestBid {},
    #[returns(TotalBidResponse)]
    TotalBid { addr: String },
}

#[cw_serde]
pub enum ExecuteMsg {
    Bid {},
    Close {},
    Retract { receiver: Option<String> },
}

#[cw_serde]
#[derive(Default)]
pub struct HighestBidResponse {
    pub bid_closed: bool,
    pub winner: Option<String>,
    pub bidder: Option<String>,
    pub amount: Option<Coin>,
}

#[cw_serde]
#[derive(Default)]
pub struct TotalBidResponse {
    pub bid_closed: bool,
    pub amount: Option<Coin>,
}

impl HighestBidResponse {
    pub fn with_winner(mut self, winner: impl Into<String>) -> Self {
        self.winner = Some(winner.into());
        self
    }

    pub fn with_bidder(mut self, bidder: impl Into<String>, amount: Coin) -> Self {
        self.bidder = Some(bidder.into());
        self.amount = Some(amount);
        self
    }
}
