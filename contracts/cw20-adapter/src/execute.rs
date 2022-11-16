use cosmwasm_std::{Addr, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, to_binary, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use injective_cosmwasm::{create_burn_tokens_msg, create_mint_tokens_msg, create_new_denom_msg, InjectiveMsgWrapper, InjectiveQueryWrapper};

use crate::common::{denom_parser, get_cw20_address_from_denom, get_denom, query_denom_creation_fee};
use crate::error::ContractError;
use crate::state::CW20_CONTRACTS;

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
    for required_coin in required_funds {
        let pf = provided_funds
            .find(|c| -> bool { c.denom == required_coin.denom })
            .ok_or(ContractError::NotEnoughBalanceToPayDenomCreationFee)?;
        if pf.amount < required_coin.amount {
            return Err(ContractError::NotEnoughBalanceToPayDenomCreationFee);
        } else if pf.amount > required_coin.amount {
            return Err(ContractError::SuperfluousFundsProvided);
        }
    }

    let create_denom_msg = register_contract_and_get_message(deps, &env, &addr)?;
    Ok(Response::new().add_message(create_denom_msg))
}

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

pub fn handle_redeem_msg(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
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
    let recipient = recipient.unwrap_or_else(|| info.sender.to_string());

    let cw20_transfer_message = WasmMsg::Execute {
        contract_addr: cw20_addr,
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient,
            amount: tokens_to_exchange.amount,
        })?,
        funds: vec![],
    };
    let burn_tf_tokens_message = create_burn_tokens_msg(env.contract.address, tokens_to_exchange);
    Ok(Response::new().add_message(cw20_transfer_message).add_message(burn_tf_tokens_message))
}

fn is_contract_registered(deps: &DepsMut<InjectiveQueryWrapper>, addr: &Addr) -> bool {
    CW20_CONTRACTS.contains(deps.storage, addr.as_ref())
}

fn ensure_sufficient_create_denom_balance(deps: &DepsMut<InjectiveQueryWrapper>, env: &Env) -> Result<(), ContractError> {
    let required_funds = query_denom_creation_fee(&deps.querier)?;

    for c in required_funds {
        let balance = deps.querier.query_balance(env.contract.address.as_str(), c.denom)?;
        if balance.amount < c.amount {
            return Err(ContractError::NotEnoughBalanceToPayDenomCreationFee);
        }
    }
    Ok(())
}

fn register_contract_and_get_message(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: &Env,
    addr: &Addr,
) -> Result<CosmosMsg<InjectiveMsgWrapper>, ContractError> {
    let contract_address = addr.to_string();
    CW20_CONTRACTS.insert(deps.storage, &contract_address)?;
    let create_denom_message = create_new_denom_msg(env.contract.address.to_string(), contract_address);

    Ok(create_denom_message)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        Addr,
        BalanceResponse,
        Coin, ContractResult, CosmosMsg, from_binary, QuerierResult, SubMsg, SystemError, SystemResult, testing::{mock_env, mock_info}, to_binary, Uint128, WasmMsg,
    };
    use cw20::Cw20ExecuteMsg;
    use injective_cosmwasm::{
        HandlesBankBalanceQuery, HandlesFeeQuery, InjectiveMsg, InjectiveMsgWrapper, InjectiveRoute, mock_dependencies, WasmMockQuerier,
    };

    use {handle_on_received_cw20_funds_msg, handle_redeem_msg, handle_register_msg};
    use ContractError;
    use CW20_CONTRACTS;

    use super::*;

    const CONTRACT_ADDRESS: &str = "inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw";
    const CW_20_ADDRESS: &str = "inj1pjcw9hhx8kf462qtgu37p7l7shyqgpfr82r6em";
    const SENDER: &str = "inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h";

    #[test]
    fn it_handles_correct_register_msg_with_exact_funds() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);
        let response = handle_register_msg(
            deps.as_mut(),
            env,
            mock_info(SENDER, &[Coin::new(10, "inj")]),
            Addr::unchecked(CW_20_ADDRESS),
        )
            .unwrap();

        let contract_registered = CW20_CONTRACTS.contains(&deps.storage, CW_20_ADDRESS);
        assert!(contract_registered, "contract wasn't registered");

        assert_eq!(response.messages.len(), 1, "incorrect number of messages returned");

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
    }

    #[test]
    fn it_handles_correct_register_msg_with_extra_funds() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);
        let response_err = handle_register_msg(
            deps.as_mut(),
            env,
            mock_info(SENDER, &[Coin::new(100, "inj"), Coin::new(20, "usdt")]),
            Addr::unchecked(CW_20_ADDRESS),
        )
            .unwrap_err();
        assert_eq!(response_err, ContractError::SuperfluousFundsProvided);
    }

    #[test]
    fn it_handles_correct_register_msg_with_non_cannonical_cw20_address() {
        let mut deps = mock_dependencies();
        let non_cannonical_address = "stefan";
        let response = handle_register_msg(
            deps.as_mut(),
            mock_env(),
            mock_info(SENDER, &[Coin::new(10, "inj")]),
            Addr::unchecked(non_cannonical_address.to_string()),
        )
            .unwrap();

        let contract_registered = CW20_CONTRACTS.contains(&deps.storage, non_cannonical_address);
        assert!(contract_registered, "contract wasn't registered");

        assert_eq!(response.messages.len(), 1, "incorrect number of messages returned");

        if let SubMsg {
            msg: CosmosMsg::Custom(InjectiveMsgWrapper { route, msg_data }),
            ..
        } = response.messages.first().unwrap()
        {
            assert_eq!(route, &InjectiveRoute::Tokenfactory, "submessage had wrong route");
            if let InjectiveMsg::CreateDenom { sender, subdenom } = msg_data {
                assert_eq!("cosmos2contract", sender.as_str(), "incorrect sender in the create denom message");
                assert_eq!(
                    non_cannonical_address,
                    subdenom.as_str(),
                    "incorrect subdenom in the create denom message"
                );
            } else {
                panic!("incorrect injective message found")
            }
        } else {
            panic!("incorrect submessage type found")
        }
    }

    #[test]
    fn it_returns_error_if_already_registered_register_msg() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let contract_address = Addr::unchecked("amazing_address");
        CW20_CONTRACTS.insert(storage, contract_address.as_str()).unwrap();

        let response = handle_register_msg(deps.as_mut(), mock_env(), mock_info("sender", &[]), contract_address);

        assert_eq!(
            response.unwrap_err(),
            ContractError::ContractAlreadyRegistered,
            "incorrect error returned"
        )
    }

    #[test]
    fn it_returns_error_if_cannot_query_denom_creation_fee_register_msg() {
        let mut deps = mock_dependencies();
        deps.querier = WasmMockQuerier {
            token_factory_denom_creation_fee_handler: create_denom_creation_fee_failing_handler(),
            ..Default::default()
        };

        let response = handle_register_msg(
            deps.as_mut(),
            mock_env(),
            mock_info(SENDER, &[Coin::new(10, "inj")]),
            Addr::unchecked(CW_20_ADDRESS),
        )
            .unwrap_err();

        assert!(response.to_string().contains("custom error"), "incorrect error returned");

        let contract_registered = CW20_CONTRACTS.contains(&deps.storage, CW_20_ADDRESS);
        assert!(!contract_registered, "contract was registered");
    }

    #[test]
    fn it_returns_error_if_mismatched_denom_is_passed_register_msg() {
        let mut deps = mock_dependencies();
        let response = handle_register_msg(
            deps.as_mut(),
            mock_env(),
            mock_info(SENDER, &[Coin::new(10, "usdt")]),
            Addr::unchecked(CW_20_ADDRESS),
        )
            .unwrap_err();

        assert_eq!(response, ContractError::NotEnoughBalanceToPayDenomCreationFee, "incorrect error returned");

        let contract_registered = CW20_CONTRACTS.contains(&deps.storage, CW_20_ADDRESS);
        assert!(!contract_registered, "contract was registered");
    }

    #[test]
    fn it_returns_error_if_insufficient_coins_are_passed_register_msg() {
        let mut deps = mock_dependencies();
        let response = handle_register_msg(
            deps.as_mut(),
            mock_env(),
            mock_info(SENDER, &[Coin::new(9, "inj")]),
            Addr::unchecked(CW_20_ADDRESS),
        )
            .unwrap_err();

        assert_eq!(response, ContractError::NotEnoughBalanceToPayDenomCreationFee, "incorrect error returned");

        let contract_registered = CW20_CONTRACTS.contains(&deps.storage, CW_20_ADDRESS);
        assert!(!contract_registered, "contract was registered");
    }

    #[test]
    fn it_returns_error_if_no_coins_are_passed_register_msg() {
        let mut deps = mock_dependencies();
        let response = handle_register_msg(deps.as_mut(), mock_env(), mock_info(SENDER, &[]), Addr::unchecked(CW_20_ADDRESS)).unwrap_err();

        assert_eq!(response, ContractError::NotEnoughBalanceToPayDenomCreationFee, "incorrect error returned");

        let contract_registered = CW20_CONTRACTS.contains(&deps.storage, CW_20_ADDRESS);
        assert!(!contract_registered, "contract was registered");
    }

    #[test]
    fn it_handles_receive_correctly_if_not_already_registered() {
        let mut deps = mock_dependencies();
        deps.querier = WasmMockQuerier {
            balance_query_handler: create_custom_bank_balance_query_handler(Coin::new(10, "inj")),
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
        let response =
            handle_on_received_cw20_funds_msg(deps.as_mut(), env, mock_info(CW_20_ADDRESS, &[Coin::new(1000, "usdt")]), SENDER.to_string(), amount_to_send).unwrap_err();

        let contract_registered = CW20_CONTRACTS.contains(&deps.storage, CW_20_ADDRESS);
        assert!(!contract_registered, "contract was registered");

        assert_eq!(response, ContractError::SuperfluousFundsProvided, "funds were provided");
    }

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
        )
            .unwrap_err();
        assert_eq!(response, ContractError::NoRegisteredTokensProvided, "incorrect error returned")
    }

    #[test]
    fn it_returns_error_if_redeeming_zero_tokens() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.contract.address = Addr::unchecked(CONTRACT_ADDRESS);

        let response = handle_redeem_msg(deps.as_mut(), env, mock_info(CW_20_ADDRESS, &[]), Some(SENDER.to_string())).unwrap_err();
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
        )
            .unwrap_err();
        assert_eq!(response, ContractError::NoRegisteredTokensProvided, "incorrect error returned")
    }

    fn create_denom_creation_fee_failing_handler() -> Option<Box<dyn HandlesFeeQuery>> {
        struct Temp();
        impl HandlesFeeQuery for Temp {
            fn handle(&self) -> QuerierResult {
                SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "custom error".to_string(),
                })
            }
        }
        Some(Box::new(Temp()))
    }

    fn create_custom_bank_balance_query_handler(balance: Coin) -> Option<Box<dyn HandlesBankBalanceQuery>> {
        struct Temp {
            balance: Coin,
        }
        impl HandlesBankBalanceQuery for Temp {
            fn handle(&self, _: String, _: String) -> QuerierResult {
                let response = BalanceResponse {
                    amount: Coin {
                        denom: self.balance.denom.clone(),
                        amount: self.balance.amount,
                    },
                };
                SystemResult::Ok(ContractResult::from(to_binary(&response)))
            }
        }
        Some(Box::new(Temp { balance }))
    }
}
