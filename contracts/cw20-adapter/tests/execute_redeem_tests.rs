use cosmwasm_std::{
    from_binary,
    testing::{mock_env, mock_info},
    to_binary, Addr, Coin, CosmosMsg, SubMsg, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw20_adapter::{error::ContractError, execute_redeem::handle_redeem_msg, state::CW20_CONTRACTS};
use injective_cosmwasm::{mock_dependencies, InjectiveMsg, InjectiveMsgWrapper, InjectiveRoute};

use crate::common::{CONTRACT_ADDRESS, CW_20_ADDRESS, SENDER};

mod common;

#[test]
fn it_handles_redeem_and_transfer_correctly() {
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

#[test]
fn it_handles_redeem_and_send_correctly() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);
    CW20_CONTRACTS.insert(&mut deps.storage, CW_20_ADDRESS).unwrap();

    let coins_to_burn = Coin::new(10, format!("factory/{}/{}", CONTRACT_ADDRESS, CW_20_ADDRESS));
    let response = handle_redeem_msg(
        deps.as_mut(),
        env,
        mock_info(CW_20_ADDRESS, &[coins_to_burn.clone()]),
        Some(CW_20_ADDRESS.to_string()),
        Some(to_binary(&coins_to_burn).unwrap()), // doesn't matter what is the message
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

        if let Cw20ExecuteMsg::Send { contract, amount, .. } = deserialised_msg {
            assert_eq!(CW_20_ADDRESS, contract.as_str(), "incorrect recipient in the transfer message");
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
