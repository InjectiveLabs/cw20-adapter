use cosmwasm_std::{to_binary, Binary, DepsMut, Env, MessageInfo, Response, WasmMsg};
use cw20::Cw20ExecuteMsg;
use injective_cosmwasm::{create_burn_tokens_msg, InjectiveMsgWrapper, InjectiveQueryWrapper};

use crate::common::{AdapterCoin, AdapterDenom};
use crate::error::ContractError;
use crate::state::CW20_CONTRACTS;

pub fn handle_redeem_msg(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
    submessage: Option<Binary>,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    let recipient = recipient.unwrap_or_else(|| info.sender.to_string());

    if info.funds.len() > 1 {
        return Err(ContractError::SuperfluousFundsProvided);
    }
    let tokens_to_exchange = info
        .funds
        .iter()
        .find_map(|c| -> Option<AdapterCoin> {
            match AdapterDenom::new(&c.denom) {
                Ok(denom) => Some(AdapterCoin { amount: c.amount, denom }),
                Err(_) => None,
            }
            // if denom_parser.is_match(&c.denom) {
            //     Some(c.clone())
            // } else {
            //     None
            // }
        })
        .ok_or(ContractError::NoRegisteredTokensProvided)?;

    let cw20_addr = tokens_to_exchange.denom.cw20_addr.clone();
    // let cw20_addr = get_cw20_address_from_denom(&denom_parser, &tokens_to_exchange.denom).ok_or(ContractError::NoRegisteredTokensProvided)?;
    let is_contract_registered = CW20_CONTRACTS.contains(deps.storage, &tokens_to_exchange.denom.cw20_addr);
    if !is_contract_registered {
        return Err(ContractError::NoRegisteredTokensProvided);
    }

    let burn_tf_tokens_message = create_burn_tokens_msg(env.contract.address, tokens_to_exchange.as_coin());

    let cw20_message: WasmMsg = match submessage {
        None => WasmMsg::Execute {
            contract_addr: cw20_addr,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient,
                amount: tokens_to_exchange.amount,
            })?,
            funds: vec![],
        },
        Some(msg) => WasmMsg::Execute {
            contract_addr: cw20_addr,
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: recipient,
                amount: tokens_to_exchange.amount,
                msg,
            })?,
            funds: vec![],
        },
    };
    Ok(Response::new().add_message(cw20_message).add_message(burn_tf_tokens_message))
}
