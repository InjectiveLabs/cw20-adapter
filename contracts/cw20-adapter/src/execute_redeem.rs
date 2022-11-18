

use cosmwasm_std::{Binary, Coin, DepsMut, Env, MessageInfo, Response, to_binary, WasmMsg};
use cw20::{Cw20ExecuteMsg};
use injective_cosmwasm::{create_burn_tokens_msg, InjectiveMsgWrapper, InjectiveQueryWrapper};

use crate::common::{denom_parser, get_cw20_address_from_denom};
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
    let denom_parser = denom_parser();
    let tokens_to_exchange = info
        .funds
        .iter()
        .find_map(|c| -> Option<Coin> {
            if denom_parser.is_match(&c.denom) {
                Some(c.clone())
            } else {
                None
            }
        })
        .ok_or(ContractError::NoRegisteredTokensProvided)?;

    let cw20_addr = get_cw20_address_from_denom(&denom_parser, &tokens_to_exchange.denom).ok_or(ContractError::NoRegisteredTokensProvided)?;
    let contract_registered = CW20_CONTRACTS.contains(deps.storage, cw20_addr.as_str());
    if !contract_registered {
        return Err(ContractError::NoRegisteredTokensProvided);
    }

    let burn_tf_tokens_message = create_burn_tokens_msg(env.contract.address, tokens_to_exchange.clone());

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
            contract_addr: cw20_addr.clone(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: cw20_addr,
                amount: tokens_to_exchange.amount,
                msg,
            })?,
            funds: vec![],
        },
    };
    Ok(Response::new().add_message(cw20_message).add_message(burn_tf_tokens_message))
}


#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        Addr,
        Coin, CosmosMsg, from_binary, SubMsg, testing::{mock_env, mock_info}, WasmMsg,
    };
    use cw20::Cw20ExecuteMsg;
    use injective_cosmwasm::{
        InjectiveMsg, InjectiveMsgWrapper, InjectiveRoute, mock_dependencies,
    };

    use {handle_redeem_msg};
    use ContractError;

    use crate::common::test_utils::{CONTRACT_ADDRESS, CW_20_ADDRESS, SENDER};
    

    use super::*;

    #[test]
    fn it_handles_redeem_correctly() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);
        CW20_CONTRACTS.insert(&mut deps.storage, CW_20_ADDRESS).unwrap();

        let coins_to_burn = Coin::new(10, format!("factory/{}/{}", CONTRACT_ADDRESS, CW_20_ADDRESS));
        let response = handle_redeem_msg(
            deps.as_mut(),
            env,
            mock_info(CW_20_ADDRESS, &[coins_to_burn.clone()]),
            Some(SENDER.to_string()),
            None,
        )
            .unwrap();

        assert_eq!(response.messages.len(), 2, "incorrect number of messages returned");

        if let SubMsg {
            msg: CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, funds }),
            ..
        } = response.messages.first().unwrap()
        {
            let expected_coins: Vec<Coin> = vec![];
            assert_eq!(&expected_coins, funds, "incorrect funds found in execute message");
            assert_eq!(CW_20_ADDRESS, contract_addr, "incorrect contact_addr in execute message");

            let deserialised_msg: Cw20ExecuteMsg = from_binary(msg).unwrap();

            if let Cw20ExecuteMsg::Transfer { recipient, amount } = deserialised_msg {
                assert_eq!(SENDER, recipient.as_str(), "incorrect recipient in the transfer message");
                assert_eq!(coins_to_burn.amount, amount, "incorrect amount in the transfer message");
            } else {
                panic!("incorrect injective message found")
            }
        } else {
            panic!("incorrect submessage type found")
        }

        if let SubMsg {
            msg: CosmosMsg::Custom(InjectiveMsgWrapper { route, msg_data }),
            ..
        } = response.messages.get(1).unwrap()
        {
            assert_eq!(route, &InjectiveRoute::Tokenfactory, "submessage had wrong route");
            if let InjectiveMsg::Burn { sender, amount } = msg_data {
                assert_eq!(CONTRACT_ADDRESS, sender.as_str(), "incorrect sender in the create denom message");
                assert_eq!(&coins_to_burn, amount, "incorrect amount in the burn message");
            } else {
                panic!("incorrect injective message found")
            }
        } else {
            panic!("incorrect submessage type found")
        }
    }

    #[test]
    fn it_returns_error_if_redeeming_non_factory_token() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);

        let response = handle_redeem_msg(
            deps.as_mut(),
            env,
            mock_info(CW_20_ADDRESS, &[Coin::new(10, "usdt")]),
            Some(SENDER.to_string()),
            None,
        )
            .unwrap_err();
        assert_eq!(response, ContractError::NoRegisteredTokensProvided, "incorrect error returned")
    }

    #[test]
    fn it_returns_error_if_redeeming_zero_tokens() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);

        let response = handle_redeem_msg(deps.as_mut(), env, mock_info(CW_20_ADDRESS, &[]), Some(SENDER.to_string()), None).unwrap_err();
        assert_eq!(response, ContractError::NoRegisteredTokensProvided, "incorrect error returned")
    }

    #[test]
    fn it_returns_error_if_redeeming_unknown_factory_token() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);

        let response = handle_redeem_msg(
            deps.as_mut(),
            env,
            mock_info(CW_20_ADDRESS, &[Coin::new(10, format!("factory/{}/{}", CONTRACT_ADDRESS, CW_20_ADDRESS))]),
            Some(SENDER.to_string()),
            None,
        )
            .unwrap_err();
        assert_eq!(response, ContractError::NoRegisteredTokensProvided, "incorrect error returned")
    }



}
