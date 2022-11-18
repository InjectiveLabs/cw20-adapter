use crate::common::{ensure_sufficient_create_denom_balance, get_denom, is_contract_registered, register_contract_and_get_message};
use crate::error::ContractError;
use cosmwasm_std::{Coin, DepsMut, Env, MessageInfo, Response, Uint128};
use injective_cosmwasm::{create_mint_tokens_msg, InjectiveMsgWrapper, InjectiveQueryWrapper};

pub fn handle_on_received_cw20_funds_msg(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    if !info.funds.is_empty() {
        return Err(ContractError::SuperfluousFundsProvided);
    }
    let mut response = Response::new();
    let contract = info.sender;
    if !is_contract_registered(&deps, &contract) {
        ensure_sufficient_create_denom_balance(&deps, &env)?;
        response = response.add_message(register_contract_and_get_message(deps, &env, &contract)?);
    }
    let master = env.contract.address;

    let denom = get_denom(&master, &contract);
    let coins_to_mint = Coin::new(amount.u128(), denom);
    let mint_tf_tokens_message = create_mint_tokens_msg(master, coins_to_mint, sender);

    Ok(response.add_message(mint_tf_tokens_message))
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_env, mock_info},
        Addr, Coin, CosmosMsg, SubMsg, Uint128,
    };

    use injective_cosmwasm::{mock_dependencies, InjectiveMsg, InjectiveMsgWrapper, InjectiveRoute, WasmMockQuerier};

    use ContractError;

    use crate::common::test_utils::{
        create_custom_bank_balance_query_handler, create_cw20_info_query_handler, CONTRACT_ADDRESS, CW_20_ADDRESS, SENDER,
    };
    use crate::state::CW20_CONTRACTS;

    use super::*;

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
        let response =
            handle_on_received_cw20_funds_msg(deps.as_mut(), env, mock_info(CW_20_ADDRESS, &[]), SENDER.to_string(), amount_to_send).unwrap();

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
        let response =
            handle_on_received_cw20_funds_msg(deps.as_mut(), env, mock_info(CW_20_ADDRESS, &[]), SENDER.to_string(), amount_to_send).unwrap();

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
}
