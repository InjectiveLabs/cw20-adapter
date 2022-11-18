use std::cmp::Ordering;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response};
use injective_cosmwasm::{InjectiveMsgWrapper, InjectiveQueryWrapper};
use crate::common::{is_contract_registered, query_denom_creation_fee, register_contract_and_get_message};
use crate::error::ContractError;

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

        match pf.amount.cmp(&required_coin.amount) {
            Ordering::Greater => return Err(ContractError::SuperfluousFundsProvided),
            Ordering::Less => return Err(ContractError::NotEnoughBalanceToPayDenomCreationFee),
            Ordering::Equal => {}
        }
    }

    let create_denom_msg = register_contract_and_get_message(deps, &env, &addr)?;
    Ok(Response::new().add_message(create_denom_msg))
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
        create_smart_query_handler, HandlesBankBalanceQuery, HandlesFeeQuery, InjectiveMsg, InjectiveMsgWrapper, InjectiveRoute, mock_dependencies,
        WasmMockQuerier,
    };

    use handle_register_msg;
    use ContractError;

    use crate::common::test_utils::{create_cw20_failing_info_query_handler, create_cw20_info_query_handler, create_denom_creation_fee_failing_handler};
    use crate::state::CW20_CONTRACTS;

    use super::*;

    const CONTRACT_ADDRESS: &str = "inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw";
    const CW_20_ADDRESS: &str = "inj1pjcw9hhx8kf462qtgu37p7l7shyqgpfr82r6em";
    const SENDER: &str = "inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h";

    #[test]
    fn it_handles_correct_register_msg_with_exact_funds() {
        let mut deps = mock_dependencies();
        deps.querier = WasmMockQuerier {
            smart_query_handler: create_cw20_info_query_handler(),
            ..Default::default()
        };

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
        deps.querier = WasmMockQuerier {
            smart_query_handler: create_cw20_failing_info_query_handler(),
            ..Default::default()
        };

        let non_cannonical_address = "stefan";
        let response = handle_register_msg(
            deps.as_mut(),
            mock_env(),
            mock_info(SENDER, &[Coin::new(10, "inj")]),
            Addr::unchecked(non_cannonical_address.to_string()),
        )
            .unwrap_err();

        assert_eq!(ContractError::NotCw20Address, response, "should fail with wrong cw-20 address");

        let contract_registered = CW20_CONTRACTS.contains(&deps.storage, non_cannonical_address);
        assert!(!contract_registered, "contract was registered");
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
    fn it_returns_error_if_register_is_not_cw20_msg() {
        let mut deps = mock_dependencies();
        deps.querier = WasmMockQuerier {
            smart_query_handler: create_cw20_failing_info_query_handler(),
            ..Default::default()
        };

        let response = handle_register_msg(deps.as_mut(), mock_env(), mock_info(SENDER, &vec![Coin::new(10, "inj")]), Addr::unchecked(CW_20_ADDRESS)).unwrap_err();

        assert_eq!(response, ContractError::NotCw20Address, "incorrect error returned");

        let contract_registered = CW20_CONTRACTS.contains(&deps.storage, CW_20_ADDRESS);
        assert!(!contract_registered, "contract was registered");
    }
}