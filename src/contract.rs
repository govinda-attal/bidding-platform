use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult, Uint128};
use cw2::set_contract_version;

use crate::{
    msg::InstantiateMsg,
    state::{CommissionParams, BID_DENOM, BID_OPEN, COMMISSION_PARAMS, HIGHEST_BID, ITEM, OWNER},
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ITEM.save(deps.storage, &msg.item)?;
    BID_DENOM.save(deps.storage, &msg.bid_denom)?;
    HIGHEST_BID.save(deps.storage, &Uint128::new(0))?;
    COMMISSION_PARAMS.save(
        deps.storage,
        &CommissionParams {
            minimum_tokens: msg.commission_minimum_tokens.clone(),
            part: msg.commission_part.clone(),
        },
    )?;

    BID_OPEN.save(deps.storage, &true)?;

    let mut owner = info.sender.clone();
    if let Some(specified_owner) = msg.owner {
        owner = deps.api.addr_validate(&specified_owner)?;
    }

    OWNER.save(deps.storage, &owner)?;
    let resp = Response::new()
        .add_attribute("action", "new_item")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("item", msg.item)
        .add_attribute("owner", owner.as_str());

    Ok(resp)
}

pub mod query {
    use cosmwasm_std::{Coin, Deps, StdResult, Uint128};

    use crate::{
        msg::{HighestBidResponse, TotalBidResponse},
        state::{BIDS, BID_DENOM, BID_OPEN, HIGHEST_BID, HIGHEST_BIDDER},
    };

    pub fn highest_bid(deps: Deps) -> StdResult<HighestBidResponse> {
        let bid_open = BID_OPEN.load(deps.storage)?;
        let highest_bid = HIGHEST_BID.load(deps.storage)?;
        let mut resp = HighestBidResponse::default();

        if highest_bid > Uint128::new(0) {
            resp.amount = Some(Coin {
                denom: BID_DENOM.load(deps.storage)?,
                amount: highest_bid,
            });
            let highest_bidder = HIGHEST_BIDDER.may_load(deps.storage)?;
            if let Some(highest_bidder) = highest_bidder {
                resp.bidder = Some(highest_bidder.to_string())
            }
        }

        if !bid_open {
            resp.bid_closed = true;
            resp.winner = resp.bidder.clone();
        }

        Ok(resp)
    }

    pub fn total_bid(deps: Deps, addr: String) -> StdResult<TotalBidResponse> {
        let addr = deps.api.addr_validate(&addr)?;

        let mut resp = TotalBidResponse::default();
        resp.bid_closed = !BID_OPEN.load(deps.storage)?;

        if let Some(amount) = BIDS.may_load(deps.storage, addr)? {
            resp.amount = Some(Coin {
                denom: BID_DENOM.load(deps.storage)?,
                amount,
            })
        }

        Ok(resp)
    }
}

pub mod execute {
    use std::ops::Mul;

    use crate::{
        error::ContractError,
        state::{BIDS, BID_DENOM, BID_OPEN, COMMISSION_PARAMS, HIGHEST_BID, HIGHEST_BIDDER, OWNER},
    };
    use cosmwasm_std::{
        coins, ensure, BankMsg, Coin, Decimal, DepsMut, Env, MessageInfo, Response, Uint128,
    };

    pub fn bid(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        ensure!(BID_OPEN.load(deps.storage)?, ContractError::BidClosed);
        ensure!(
            info.sender != OWNER.load(deps.storage)?,
            ContractError::OwnerCannotBid
        );

        let bid_denom = BID_DENOM.load(deps.storage)?;
        let bid_funds = info.funds.into_iter().find(|coin| coin.denom == bid_denom);
        ensure!(
            bid_funds.is_some(),
            ContractError::BidRejectedMissingTokensInDenom { denom: bid_denom }
        );

        let mut amount = bid_funds.unwrap().amount;
        let commission_params = COMMISSION_PARAMS.load(deps.storage)?;
        let mut commission_amt = commission_params
            .part
            .mul(Decimal::new(amount))
            .to_uint_ceil();

        if commission_amt < commission_params.minimum_tokens {
            commission_amt = commission_params.minimum_tokens;
        }
        amount = amount - commission_amt;

        if let Some(prev_total_amount) = BIDS.may_load(deps.storage, info.sender.clone())? {
            amount += prev_total_amount;
        }

        let highest_bid_amount = HIGHEST_BID.load(deps.storage)?;
        ensure!(
            amount > highest_bid_amount,
            ContractError::BidRejected {
                highest_bid: Coin {
                    amount: highest_bid_amount,
                    denom: bid_denom
                }
            }
        );

        HIGHEST_BIDDER.save(deps.storage, &info.sender)?;
        HIGHEST_BID.save(deps.storage, &amount)?;
        BIDS.save(deps.storage, info.sender.clone(), &amount)?;

        let mut resp = Response::new();
        if commission_amt.gt(&Uint128::new(0)) {
            let commission_msg = BankMsg::Send {
                to_address: OWNER.load(deps.storage)?.to_string(),
                amount: coins(commission_amt.into(), bid_denom.clone()),
            };
            resp = resp.add_message(commission_msg)
        }

        resp = resp
            .add_attribute("action", "bid")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("bid_denom", bid_denom)
            .add_attribute("total_bid_amount", amount.to_string());

        Ok(resp)
    }

    pub fn close(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        ensure!(BID_OPEN.load(deps.storage)?, ContractError::BidClosed);

        let owner = OWNER.load(deps.storage)?;
        ensure!(
            info.sender == owner,
            ContractError::Unauthorized {
                owner: owner.into()
            }
        );

        BID_OPEN.save(deps.storage, &false)?;

        let bid_denom = BID_DENOM.load(deps.storage)?;
        let mut resp = Response::new()
            .add_attribute("action", "close")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("bid_denom", &bid_denom);

        let closing_bid = HIGHEST_BID.load(deps.storage)?;
        if closing_bid.u128() == 0 {
            return Ok(resp);
        }

        resp = resp
            .add_attribute("closing_bid", closing_bid.to_string())
            .add_attribute("winner", HIGHEST_BIDDER.load(deps.storage)?);

        let bank_msg = BankMsg::Send {
            to_address: owner.to_string(),
            amount: coins(closing_bid.u128(), bid_denom),
        };
        resp = resp.add_message(bank_msg);

        Ok(resp)
    }

    pub fn retract(
        deps: DepsMut,
        info: MessageInfo,
        receiver: Option<String>,
    ) -> Result<Response, ContractError> {
        ensure!(!BID_OPEN.load(deps.storage)?, ContractError::BidOpen);

        let mut resp = Response::new()
            .add_attribute("action", "retract")
            .add_attribute("sender", info.sender.as_str());
        let mut beneficiary = info.sender;

        let amount = BIDS.may_load(deps.storage, beneficiary.clone())?;
        if amount.is_none() {
            return Ok(resp);
        }

        let amount = amount.unwrap();
        let bid_denom = BID_DENOM.load(deps.storage)?;

        if let Some(receiver) = receiver {
            beneficiary = deps.api.addr_validate(&receiver)?;
        }

        resp = resp
            .add_attribute("denom", &bid_denom)
            .add_attribute("amount", amount.to_string())
            .add_attribute("beneficiary", beneficiary.as_str());

        let msg = BankMsg::Send {
            to_address: beneficiary.to_string(),
            amount: coins(amount.u128(), bid_denom),
        };

        Ok(resp.add_message(msg))
    }
}
