use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, DepsMut, Env, from_binary, MessageInfo, QuerierWrapper, Response, StdError, StdResult, to_binary, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use injective_cosmwasm::{create_burn_tokens_msg, create_mint_tokens_msg, create_new_denom_msg,  InjectiveMsgWrapper, InjectiveQuerier, InjectiveQueryWrapper};

use crate::common::{get_cw20_address_from_denom, get_denom, is_token_factory_denom};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, ReceiveSubmsg};
use crate::state::CW20_CONTRACTS;

pub fn handle_register_msg(
    deps: DepsMut<InjectiveQueryWrapper>,
    info: MessageInfo,
    env: &Env,
    addr: &Addr,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    if contract_registered(&deps, addr) {
        return Err(ContractError::ContractAlreadyRegistered);
    }
    let querier = InjectiveQuerier::new(&deps.querier);
    let required_funds = querier.query_token_factory_creation_fee()?.fee;

    let mut provided_funds = info.funds.iter();
    for required_coin in required_funds {
        let pf = provided_funds.find(|c| -> bool { c.denom == required_coin.denom }).ok_or(ContractError::NotEnoughBalanceToPayDenomCreationFee)?;
        if pf.amount < required_coin.amount {
            return Err(ContractError::NotEnoughBalanceToPayDenomCreationFee);
        }
    }

    let create_denom_msg = register_contract_and_get_message(deps, env, addr)?;
    Ok(Response::new().add_message(create_denom_msg))
}

pub fn handle_on_received_cw20_funds_msg(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    let mut response = Response::new();
    let contract = info.sender;
    if !contract_registered(&deps, &contract) {
        check_account_create_denom_funds(&deps, &env)?;
        response = response.add_message(register_contract_and_get_message(deps, &env, &contract)?);
    }
    let master = env.contract.address;

    let denom = get_denom(&master, &contract);
    let coins_to_mint = Coin::new(amount.u128(), denom);
    let mint_tf_tokens_message = create_mint_tokens_msg(master, coins_to_mint, sender);

    Ok(response.add_message(mint_tf_tokens_message))
}

pub fn handle_redeem_msg(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: Env,
    info: MessageInfo,
    recipient: String,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    let tokens_to_exchange = info
        .funds
        .iter()
        .find_map(|c| -> Option<Coin> {
            if is_token_factory_denom(&c.denom) {
                Some(c.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| ContractError::NoRegisteredTokensProvided)?;

    let cw20_addr = get_cw20_address_from_denom(&tokens_to_exchange.denom)
        .ok_or(ContractError::NoRegisteredTokensProvided)?;
    let contract_registered = CW20_CONTRACTS.contains(deps.storage, cw20_addr);
    if !contract_registered {
        return Err(ContractError::NoRegisteredTokensProvided);
    }

    // let transfer_msg = Cw20ExecuteMsg::Transfer{ recipient, amount: tokens_to_exchange.amount };
    let cw20_transfer_message = WasmMsg::Execute {
        contract_addr: cw20_addr.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient,
            amount: tokens_to_exchange.amount,
        })?,
        funds: vec![],
    };
    let burn_tf_tokens_message = create_burn_tokens_msg(env.contract.address, tokens_to_exchange);
    Ok(Response::new().add_message(cw20_transfer_message).add_message(burn_tf_tokens_message))
}

fn contract_registered(deps: &DepsMut<InjectiveQueryWrapper>,
                       addr: &Addr, ) -> bool {
    CW20_CONTRACTS.contains(deps.storage, &addr.to_string())
}

fn check_account_create_denom_funds(deps: &DepsMut<InjectiveQueryWrapper>,
                                    env: &Env,
) -> Result<(), ContractError> {
    let querier = InjectiveQuerier::new(&deps.querier);
    let required_funds = querier.query_token_factory_creation_fee()?.fee;

    for c in required_funds {
        let balance = deps.querier.query_balance(env.contract.address.as_str(), c.denom)?;
        if balance.amount < c.amount {
            return Err(ContractError::NotEnoughBalanceToPayDenomCreationFee);
        }
    }
    return Ok(());
}

fn register_contract_and_get_message(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: &Env,
    addr: &Addr,
) -> Result<CosmosMsg<InjectiveMsgWrapper>, ContractError> {
    let contract_address = addr.to_string();
    CW20_CONTRACTS.insert(deps.storage, &contract_address)?;
    let create_denom_message =
        create_new_denom_msg(env.contract.address.to_string(), contract_address);

    return Ok(create_denom_message);
}
