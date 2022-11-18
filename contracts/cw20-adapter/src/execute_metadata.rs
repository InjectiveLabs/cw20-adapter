use std::cmp::Ordering;

use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, TokenInfoResponse};
use injective_cosmwasm::{addr_to_bech32, create_burn_tokens_msg, create_mint_tokens_msg, create_new_denom_msg, create_set_token_metadata_msg, InjectiveMsgWrapper, InjectiveQueryWrapper};

use crate::common::{denom_parser, fetch_cw20_metadata, get_cw20_address_from_denom, get_denom, query_denom_creation_fee};
use crate::error::ContractError;
use crate::state::CW20_CONTRACTS;

pub fn handle_update_metadata(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: Env,
    cw20_addr: Addr,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    let contract_registered = CW20_CONTRACTS.contains(deps.storage, cw20_addr.as_str());
    if !contract_registered {
        return Err(ContractError::ContractNotRegistered);
    }
    let token_metadata = fetch_cw20_metadata(&deps, cw20_addr.as_str())?;

    let denom = get_denom(&env.contract.address, &cw20_addr);
    let set_metadata_message = create_set_token_metadata_msg(denom, token_metadata.name, token_metadata.symbol, token_metadata.decimals);

    Ok(Response::new().add_message(set_metadata_message))
}



#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, CosmosMsg, SubMsg};
    use cosmwasm_std::testing::mock_env;
    use injective_cosmwasm::{InjectiveMsg, InjectiveMsgWrapper, InjectiveRoute, mock_dependencies, WasmMockQuerier};
    use crate::common::get_denom;
    use crate::common::test_utils::{CONTRACT_ADDRESS, create_cw20_info_query_handler, CW_20_ADDRESS};
    use crate::error::ContractError;
    use crate::execute_metadata::handle_update_metadata;
    use crate::state::CW20_CONTRACTS;

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
            if let InjectiveMsg::SetTokenMetadata {  denom, name, symbol, decimals }  = msg_data {
                assert_eq!(get_denom(&Addr::unchecked(CONTRACT_ADDRESS), &Addr::unchecked(CW_20_ADDRESS)), denom.as_str(), "incorrect denom in set metadata message");
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
        let mut env = mock_env();

        let err_response = handle_update_metadata(deps.as_mut(), env, Addr::unchecked(CW_20_ADDRESS)).unwrap_err();
        assert_eq!(err_response, ContractError::ContractNotRegistered, "incorrect error");
    }
}