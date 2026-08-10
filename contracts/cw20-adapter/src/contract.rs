use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use injective_cosmwasm::{InjectiveMsgWrapper, InjectiveQueryWrapper};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::{error::ContractError, execute_metadata, execute_receive, execute_redeem, execute_register, query};

pub const CONTRACT_NAME: &str = "crates.io:inj-cw20-adapter";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut<InjectiveQueryWrapper>,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response<InjectiveMsgWrapper>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

#[entry_point]
pub fn migrate(deps: DepsMut<InjectiveQueryWrapper>, _env: Env, _msg: MigrateMsg) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    let previous_version = cw2::ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("from_version", previous_version.to_string())
        .add_attribute("to_version", CONTRACT_VERSION))
}

#[entry_point]
pub fn execute(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    match msg {
        ExecuteMsg::RegisterCw20Contract { addr } => execute_register::handle_register_msg(deps, env, info, addr),
        ExecuteMsg::Receive { sender, amount, msg: _ } => execute_receive::handle_on_received_cw20_funds_msg(deps, env, info, sender, amount),
        ExecuteMsg::RedeemAndTransfer { recipient } => execute_redeem::handle_redeem_msg(deps, env, info, recipient, None),
        ExecuteMsg::RedeemAndSend { recipient, submsg } => execute_redeem::handle_redeem_msg(deps, env, info, Some(recipient), Some(submsg)),
        ExecuteMsg::UpdateMetadata { addr } => execute_metadata::handle_update_metadata(deps, env, addr),
    }
}

#[entry_point]
pub fn query(deps: Deps<InjectiveQueryWrapper>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::RegisteredContracts {} => to_json_binary(&query::registered_contracts(deps)?),
        QueryMsg::NewDenomFee {} => to_json_binary(&query::new_denom_fee(deps)?),
    }
}
