use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, DepsMut, Env, from_binary, MessageInfo, Response, StdError, StdResult, to_binary, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use injective_cosmwasm::{create_burn_tokens_msg, create_mint_tokens_msg, create_new_denom_msg, InjectiveMsgWrapper};

use crate::common::{get_cw20_address_from_denom, get_denom, is_token_factory_denom};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, ReceiveSubmsg};
use crate::state::CW20_CONTRACTS;

pub fn register(deps: DepsMut,
                env: &Env,
                addr: &Addr, ) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    let contract_address = addr.to_string();
    if CW20_CONTRACTS.contains(deps.storage, &contract_address) {
        return Err(ContractError::ContractAlreadyRegistered);
    }
    CW20_CONTRACTS.insert(deps.storage, &contract_address)?;
    let create_denom_message = create_new_denom_msg(env.contract.address.to_string(), contract_address);
    // TODO: check if there's enough funds

    Ok(Response::new().add_message(create_denom_message))
}

pub fn on_received_cw20_funds(deps: DepsMut,
                              env : Env,
                              info: MessageInfo,
                              sender: String,
                              amount: Uint128,
                              msg: Binary) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    // TODO: check if sender contains CW-20 address or user address
    let contract = info.sender;
    if !CW20_CONTRACTS.contains(deps.storage, &contract.as_str()) {
        register(deps, &env, &contract)?;
    }
    let master = env.contract.address;
    // let recipient_submsg = from_binary::<ReceiveSubmsg>(&msg)?;
    // let recipient = recipient_submsg.recipient;
    let recipient = sender;

    let denom = get_denom(&master, &contract);
    let coins_to_mint = Coin::new(amount.u128(), denom );
    let mint_tf_tokens_message = create_mint_tokens_msg(master, coins_to_mint, recipient);

    Ok(Response::new().add_message(mint_tf_tokens_message))
}

pub fn redeem(
    deps: DepsMut,
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
        .ok_or_else(|| {
            ContractError::NoRegisteredTokensProvided
        })?;

    let cw20_addr = get_cw20_address_from_denom(&tokens_to_exchange.denom).ok_or(ContractError::NoRegisteredTokensProvided)?;
    let contract_registered = CW20_CONTRACTS.contains(deps.storage, cw20_addr);
    if !contract_registered {
        return Err(ContractError::NoRegisteredTokensProvided);
    }

    // let transfer_msg = Cw20ExecuteMsg::Transfer{ recipient, amount: tokens_to_exchange.amount };
    let cw20_transfer_message = WasmMsg::Execute {
        contract_addr: tokens_to_exchange.denom.clone(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer { recipient, amount: tokens_to_exchange.amount })?,
        funds: vec![],
    };
    let burn_tf_tokens_message = create_burn_tokens_msg(env.contract.address, tokens_to_exchange);
    Ok(Response::new().add_message(cw20_transfer_message).add_message(burn_tf_tokens_message))
}