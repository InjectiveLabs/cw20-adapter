mod common;

use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, CosmosMsg, SubMsg};
use cw20_adapter::common::get_denom;
use cw20_adapter::error::ContractError;
use cw20_adapter::execute_metadata::handle_update_metadata;
use cw20_adapter::state::CW20_CONTRACTS;
use injective_cosmwasm::{mock_dependencies, InjectiveMsg, InjectiveMsgWrapper, InjectiveRoute, WasmMockQuerier};

use crate::common::{create_cw20_info_query_handler, CONTRACT_ADDRESS, CW_20_ADDRESS};

#[test]
fn it_updates_metadata() {
    let mut deps = mock_dependencies();
    deps.querier = WasmMockQuerier {
        smart_query_handler: create_cw20_info_query_handler(),
        ..Default::default()
    };
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);
    CW20_CONTRACTS.insert(&mut deps.storage, CW_20_ADDRESS).unwrap();

    let response = handle_update_metadata(deps.as_mut(), env, Addr::unchecked(CW_20_ADDRESS)).unwrap();
    assert_eq!(response.messages.len(), 1, "incorrect number of messages returned");

    if let SubMsg {
        msg: CosmosMsg::Custom(InjectiveMsgWrapper { route, msg_data }),
        ..
    } = response.messages.get(0).unwrap()
    {
        assert_eq!(route, &InjectiveRoute::Tokenfactory, "submessage had wrong route");
        if let InjectiveMsg::SetTokenMetadata {
            denom,
            name,
            symbol,
            decimals,
        } = msg_data
        {
            assert_eq!(
                get_denom(&Addr::unchecked(CONTRACT_ADDRESS), &Addr::unchecked(CW_20_ADDRESS)),
                denom.as_str(),
                "incorrect denom in set metadata message"
            );
            assert_eq!("SOL", symbol.as_str(), "incorrect symbol in set metadata message");
            assert_eq!("Solana", name.as_str(), "incorrect name in set metadata message");
            assert_eq!(6, *decimals, "incorrect decimals in set metadata message");
        } else {
            panic!("incorrect injective message found")
        }
    } else {
        panic!("incorrect submessage type found")
    }
}

#[test]
fn it_tries_to_update_not_registered_contract() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let err_response = handle_update_metadata(deps.as_mut(), env, Addr::unchecked(CW_20_ADDRESS)).unwrap_err();
    assert_eq!(err_response, ContractError::ContractNotRegistered, "incorrect error");
}
