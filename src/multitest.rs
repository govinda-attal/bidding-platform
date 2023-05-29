use cosmwasm_std::{Addr, Coin, Decimal, StdResult};
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::{
    error::ContractError,
    execute, instantiate,
    msg::{ExecuteMsg, HighestBidResponse, InstantiateMsg, QueryMsg, TotalBidResponse},
    query,
};

#[cfg(test)]
mod tests;

pub struct BiddingContract(Addr);

impl BiddingContract {
    #[track_caller]
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    #[track_caller]
    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate<'a>(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        label: &str,
        item: impl Into<String>,
        bid_denom: impl Into<String>,
        owner: impl Into<Option<&'a Addr>>,
        commission_minimum_tokens: u128,
        commission_part: Decimal,
    ) -> StdResult<BiddingContract> {
        let owner = owner.into();
        let item = item.into();
        let bid_denom = bid_denom.into();
        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                owner: owner.map(Addr::to_string),
                item,
                bid_denom,
                commission_minimum_tokens: commission_minimum_tokens.into(),
                commission_part,
            },
            &[],
            label,
            None,
        )
        .map_err(|err| err.downcast().unwrap())
        .map(BiddingContract)
    }

    #[track_caller]
    pub fn query_highest_bid(&self, app: &App) -> StdResult<HighestBidResponse> {
        app.wrap()
            .query_wasm_smart(self.addr().clone(), &QueryMsg::HighestBid {})
    }

    #[track_caller]
    pub fn query_total_bid(&self, app: &App, addr: &Addr) -> StdResult<TotalBidResponse> {
        app.wrap().query_wasm_smart(
            self.addr().to_string(),
            &QueryMsg::TotalBid {
                addr: addr.to_string(),
            },
        )
    }

    #[track_caller]
    pub fn bid(&self, app: &mut App, bidder: &Addr, tokens: Coin) -> Result<(), ContractError> {
        app.execute_contract(
            bidder.clone(),
            self.addr().clone(),
            &ExecuteMsg::Bid {},
            &[tokens],
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())
        .map(|_| ())
    }

    #[track_caller]
    pub fn close(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.addr().clone(),
            &ExecuteMsg::Close {},
            &[],
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())
        .map(|_| ())
    }

    #[track_caller]
    pub fn retract(
        &self,
        app: &mut App,
        sender: &Addr,
        receiver: Option<String>,
    ) -> Result<(), ContractError> {
        app.execute_contract(
            sender.clone(),
            self.addr().clone(),
            &ExecuteMsg::Retract { receiver },
            &[],
        )
        .map_err(|err| err.downcast::<ContractError>().unwrap())
        .map(|_| ())
    }
}

impl From<BiddingContract> for Addr {
    fn from(value: BiddingContract) -> Self {
        value.0
    }
}
