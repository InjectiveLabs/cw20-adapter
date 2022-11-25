use crate::common::{is_contract_registered, query_denom_creation_fee, register_contract_and_get_message};
use crate::error::ContractError;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response};
use injective_cosmwasm::{InjectiveMsgWrapper, InjectiveQueryWrapper};
use std::cmp::Ordering;

pub fn handle_register_msg(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: Env,
    info: MessageInfo,
    addr: Addr,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    if is_contract_registered(&deps, &addr) {
        return Err(ContractError::ContractAlreadyRegistered);
    }
    let required_funds = query_denom_creation_fee(&deps.querier)?;
    if info.funds.len() > required_funds.len() {
        return Err(ContractError::SuperfluousFundsProvided);
    }

    let mut provided_funds = info.funds.iter();

    for required_coin in &required_funds {
        let pf = provided_funds
            .find(|c| -> bool { c.denom == required_coin.denom })
            .ok_or(ContractError::NotEnoughBalanceToPayDenomCreationFee)?;

        match pf.amount.cmp(&required_coin.amount) {
            Ordering::Greater => return Err(ContractError::SuperfluousFundsProvided),
            Ordering::Less => return Err(ContractError::NotEnoughBalanceToPayDenomCreationFee),
            Ordering::Equal => {}
        }
    }

    let create_denom_msg = register_contract_and_get_message(deps, &env, &addr)?;
    Ok(Response::new().add_message(create_denom_msg))
}
