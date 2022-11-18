mod common;

use cosmwasm_std::{
    testing::{mock_env, mock_info},
    Addr, Coin, CosmosMsg, SubMsg, Uint128,
};

use cw20_adapter::{error::ContractError, execute_receive::handle_on_received_cw20_funds_msg, state::CW20_CONTRACTS};
use injective_cosmwasm::{mock_dependencies, InjectiveMsg, InjectiveMsgWrapper, InjectiveRoute, WasmMockQuerier};

use crate::common::{create_custom_bank_balance_query_handler, create_cw20_info_query_handler, CONTRACT_ADDRESS, CW_20_ADDRESS, SENDER};

#[test]
fn it_handles_receive_correctly_if_not_already_registered() {
    let mut deps = mock_dependencies();
    deps.querier = WasmMockQuerier {
        balance_query_handler: create_custom_bank_balance_query_handler(Coin::new(10, "inj")),
        smart_query_handler: create_cw20_info_query_handler(),
        ..Default::default()
    };
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);
    let amount_to_send = Uint128::new(100);
    let response = handle_on_received_cw20_funds_msg(deps.as_mut(), env, mock_info(CW_20_ADDRESS, &[]), SENDER.to_string(), amount_to_send).unwrap();

    let contract_registered = CW20_CONTRACTS.contains(&deps.storage, CW_20_ADDRESS);
    assert!(contract_registered, "contract wasn't registered");

    assert_eq!(response.messages.len(), 2, "incorrect number of messages returned");

    if let SubMsg {
        msg: CosmosMsg::Custom(InjectiveMsgWrapper { route, msg_data }),
        ..
    } = response.messages.first().unwrap()
    {
        assert_eq!(route, &InjectiveRoute::Tokenfactory, "submessage had wrong route");
        if let InjectiveMsg::CreateDenom { sender, subdenom } = msg_data {
            assert_eq!(CONTRACT_ADDRESS, sender.as_str(), "incorrect sender in the create denom message");
            assert_eq!(CW_20_ADDRESS, subdenom.as_str(), "incorrect subdenom in the create denom message");
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
        if let InjectiveMsg::Mint { sender, amount, mint_to } = msg_data {
            assert_eq!(CONTRACT_ADDRESS, sender.as_str(), "incorrect sender in the mint message");
            assert_eq!(amount_to_send, amount.amount, "incorrect amount in the mint message");
            assert_eq!(
                format!("factory/{}/{}", CONTRACT_ADDRESS, CW_20_ADDRESS),
                amount.denom,
                "incorrect amount in the mint message"
            );
            assert_eq!(SENDER, mint_to, "incorrect mint_to in the mint message");
        } else {
            panic!("incorrect injective message found")
        }
    } else {
        panic!("incorrect submessage type found")
    }
}

#[test]
fn it_handles_receive_correctly_if_already_registered() {
    let mut deps = mock_dependencies();
    deps.querier = WasmMockQuerier {
        balance_query_handler: create_custom_bank_balance_query_handler(Coin::new(10, "inj")),
        ..Default::default()
    };
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);
    CW20_CONTRACTS.insert(&mut deps.storage, CW_20_ADDRESS).unwrap();
    let amount_to_send = Uint128::new(100);
    let response = handle_on_received_cw20_funds_msg(deps.as_mut(), env, mock_info(CW_20_ADDRESS, &[]), SENDER.to_string(), amount_to_send).unwrap();

    let contract_registered = CW20_CONTRACTS.contains(&deps.storage, CW_20_ADDRESS);
    assert!(contract_registered, "contract wasn't registered");

    assert_eq!(response.messages.len(), 1, "incorrect number of messages returned");

    if let SubMsg {
        msg: CosmosMsg::Custom(InjectiveMsgWrapper { route, msg_data }),
        ..
    } = response.messages.first().unwrap()
    {
        assert_eq!(route, &InjectiveRoute::Tokenfactory, "submessage had wrong route");
        if let InjectiveMsg::Mint { sender, amount, mint_to } = msg_data {
            assert_eq!(CONTRACT_ADDRESS, sender.as_str(), "incorrect sender in the mint message");
            assert_eq!(amount_to_send, amount.amount, "incorrect amount in the mint message");
            assert_eq!(
                format!("factory/{}/{}", CONTRACT_ADDRESS, CW_20_ADDRESS),
                amount.denom,
                "incorrect amount in the mint message"
            );
            assert_eq!(SENDER, mint_to, "incorrect mint_to in the mint message");
        } else {
            panic!("incorrect injective message found")
        }
    } else {
        panic!("incorrect submessage type found")
    }
}

#[test]
fn it_returns_error_on_receive_if_contract_not_registered_and_contract_has_insufficient_balance() {
    let mut deps = mock_dependencies();
    deps.querier = WasmMockQuerier {
        balance_query_handler: create_custom_bank_balance_query_handler(Coin::new(9, "inj")),
        ..Default::default()
    };
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);
    let amount_to_send = Uint128::new(100);
    let response =
        handle_on_received_cw20_funds_msg(deps.as_mut(), env, mock_info(CW_20_ADDRESS, &[]), SENDER.to_string(), amount_to_send).unwrap_err();

    let contract_registered = CW20_CONTRACTS.contains(&deps.storage, CW_20_ADDRESS);
    assert!(!contract_registered, "contract was registered");

    assert_eq!(response, ContractError::NotEnoughBalanceToPayDenomCreationFee, "incorrect error returned");
}

#[test]
fn it_returns_error_on_receive_if_additional_funds_are_provided() {
    let mut deps = mock_dependencies();
    deps.querier = WasmMockQuerier {
        balance_query_handler: create_custom_bank_balance_query_handler(Coin::new(10, "inj")),
        ..Default::default()
    };
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);
    let amount_to_send = Uint128::new(100);
    let response = handle_on_received_cw20_funds_msg(
        deps.as_mut(),
        env,
        mock_info(CW_20_ADDRESS, &[Coin::new(1000, "usdt")]),
        SENDER.to_string(),
        amount_to_send,
    )
    .unwrap_err();

    let contract_registered = CW20_CONTRACTS.contains(&deps.storage, CW_20_ADDRESS);
    assert!(!contract_registered, "contract was registered");
    assert_eq!(response, ContractError::SuperfluousFundsProvided, "funds were provided");
}
