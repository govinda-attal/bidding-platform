#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use error::ContractError;
use msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

mod contract;
mod state;

pub mod error;
pub mod msg;

#[cfg(any(test, feature = "tests"))]
pub mod multitest;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        HighestBid {} => to_binary(&contract::query::highest_bid(deps)?),
        TotalBid { addr } => to_binary(&contract::query::total_bid(deps, addr)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;
    match msg {
        Bid {} => contract::execute::bid(deps, env, info),
        Close {} => contract::execute::close(deps, env, info),
        Retract { receiver } => contract::execute::retract(deps, info, receiver),
    }
}
