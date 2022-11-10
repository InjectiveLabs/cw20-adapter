use cosmwasm_std::{
    Binary, Deps, DepsMut, entry_point, Env, MessageInfo, Response, StdResult, to_binary, Empty,
};
use injective_cosmwasm::InjectiveMsgWrapper;

use crate::{error::ContractError, execute, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

pub const CONTRACT_NAME: &str = "crates.io:inj-cw20-adapter";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response<InjectiveMsgWrapper>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    match msg {
        ExecuteMsg::RegisterCw20Contract { addr } => execute::register(deps, &env, &addr),
        ExecuteMsg::Receive { sender, amount, msg } => execute::on_received_cw20_funds(deps, env, info, sender, amount, msg),
        ExecuteMsg::Redeem { recipient } => execute::redeem(deps, env, info, recipient),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::RegisteredContracts {} => to_binary(&query::registered_contracts(deps)?),
        QueryMsg::NewDenomFee {} => to_binary(&query::new_denom_fee(deps)?),
    }
}
