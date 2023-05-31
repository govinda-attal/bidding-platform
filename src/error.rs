use cosmwasm_std::{Coin, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("commission part can be between [0-25]%")]
    InvalidCommissionPart,

    #[error("Unauthorized - only {owner} can call it")]
    Unauthorized { owner: String },

    #[error("Owner of an item cannot bid on the item")]
    OwnerCannotBid,

    #[error("Bid rejected as current highest bid value is {highest_bid}")]
    BidRejected { highest_bid: Coin },

    #[error("Bid rejected as no {denom} tokens")]
    BidRejectedMissingTokensInDenom { denom: String },

    #[error("Bid closed")]
    BidClosed,

    #[error("Bid is open")]
    BidOpen,
}
