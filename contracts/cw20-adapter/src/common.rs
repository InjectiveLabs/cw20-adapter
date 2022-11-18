use cosmwasm_std::{Addr, Coin, CosmosMsg, DepsMut, Env, QuerierWrapper, StdResult};
use cw20::{Cw20QueryMsg, TokenInfoResponse};
use injective_cosmwasm::{create_new_denom_msg, InjectiveMsgWrapper, InjectiveQuerier, InjectiveQueryWrapper};
use regex::Regex;
use crate::error::ContractError;
use crate::state::CW20_CONTRACTS;

pub fn denom_parser() -> Regex {
    Regex::new(r"factory/(\w{42})/(\w{42})").unwrap()
}

pub fn get_cw20_address_from_denom(parser: &Regex, denom: &str) -> Option<String> {
    let captures = parser.captures(denom)?;
    let cw20addr = captures.get(2)?;
    Some(cw20addr.as_str().to_string())
}

pub fn get_denom(adapter_address: &Addr, cw20addr: &Addr) -> String {
    format!("factory/{}/{}", adapter_address, cw20addr)
}

pub fn query_denom_creation_fee(querier_wrapper: &QuerierWrapper<InjectiveQueryWrapper>) -> StdResult<Vec<Coin>> {
    let querier = InjectiveQuerier::new(querier_wrapper);
    Ok(querier.query_token_factory_creation_fee()?.fee)
}

pub fn fetch_cw20_metadata(deps: &DepsMut<InjectiveQueryWrapper>, addr: &str) -> Result<TokenInfoResponse, ContractError> {
    let msg = Cw20QueryMsg::TokenInfo {};
    deps.querier.query_wasm_smart(addr, &msg).map_err(|_e| ContractError::NotCw20Address)
}

pub fn ensure_address_is_cw20(deps: &DepsMut<InjectiveQueryWrapper>, addr: &str) -> Result<(), ContractError> {
    let msg = Cw20QueryMsg::TokenInfo {};
    let res: StdResult<TokenInfoResponse> = deps.querier.query_wasm_smart(addr, &msg);
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(ContractError::NotCw20Address),
    }
}


pub fn is_contract_registered(deps: &DepsMut<InjectiveQueryWrapper>, addr: &Addr) -> bool {
    CW20_CONTRACTS.contains(deps.storage, addr.as_ref())
}

pub fn ensure_sufficient_create_denom_balance(deps: &DepsMut<InjectiveQueryWrapper>, env: &Env) -> Result<(), ContractError> {
    let required_funds = query_denom_creation_fee(&deps.querier)?;

    for c in required_funds {
        let balance = deps.querier.query_balance(env.contract.address.as_str(), c.denom)?;
        if balance.amount < c.amount {
            return Err(ContractError::NotEnoughBalanceToPayDenomCreationFee);
        }
    }
    Ok(())
}

pub fn register_contract_and_get_message(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: &Env,
    addr: &Addr,
) -> Result<CosmosMsg<InjectiveMsgWrapper>, ContractError> {
    let contract_address = addr.to_string();
    ensure_address_is_cw20(&deps, &contract_address)?;
    CW20_CONTRACTS.insert(deps.storage, &contract_address)?;
    let create_denom_message = create_new_denom_msg(env.contract.address.to_string(), contract_address);

    Ok(create_denom_message)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_true_on_correct_token_factory_denom() {
        assert!(
            denom_parser().is_match("factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h"),
            "input was not treated as token factory denom"
        )
    }

    #[test]
    fn it_returns_false_for_non_token_factory_denom() {
        assert!(
            !denom_parser().is_match(".factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7"),
            "input was treated as token factory denom"
        )
    }

    #[test]
    fn it_returns_cw_20_address_for_token_factory_denom() {
        assert_eq!(
            get_cw20_address_from_denom(
                &denom_parser(),
                "factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h",
            )
                .unwrap(),
            "inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h",
            "wrong cw20 address returned"
        )
    }

    #[test]
    fn it_returns_none_cw_20_address_for_non_token_factory_denom() {
        assert!(
            get_cw20_address_from_denom(
                &denom_parser(),
                "factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7",
            )
                .is_none(),
            "cw20 address returned"
        )
    }

    #[test]
    fn it_returns_denom() {
        assert_eq!(
            get_denom(
                &Addr::unchecked("inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw".to_string()),
                &Addr::unchecked("inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h".to_string()),
            ),
            "factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h",
            "wrong denom returned"
        )
    }
}

#[cfg(test)]
pub mod test_utils {
    use cosmwasm_std::{Addr, BalanceResponse, Binary, BlockInfo, Coin, ContractInfo, ContractResult, Env, QuerierResult, SystemError, SystemResult, Timestamp, to_binary, TransactionInfo, Uint128};
    use cw20::TokenInfoResponse;
    use injective_cosmwasm::{HandlesBankBalanceQuery, HandlesFeeQuery, HandlesSmartQuery};

    pub const CONTRACT_ADDRESS: &str = "inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw";
    pub const CW_20_ADDRESS: &str = "inj1pjcw9hhx8kf462qtgu37p7l7shyqgpfr82r6em";
    pub const SENDER: &str = "inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h";

    pub fn mock_env(addr: &str) -> Env {
        Env {
            block: BlockInfo {
                height: 12_345,
                time: Timestamp::from_nanos(1_571_797_419_879_305_533),
                chain_id: "inj-testnet-14002".to_string(),
            },
            transaction: Some(TransactionInfo { index: 3 }),
            contract: ContractInfo {
                address: Addr::unchecked(addr),
            },
        }
    }

    pub fn create_cw20_info_query_handler() -> Option<Box<dyn HandlesSmartQuery>> {
        struct A();
        impl HandlesSmartQuery for A {
            fn handle(&self, _: &str, _: &Binary) -> QuerierResult {
                let response = TokenInfoResponse {
                    name: "Solana".to_string(),
                    symbol: "SOL".to_string(),
                    decimals: 6,
                    total_supply: Uint128::new(1000),
                };
                SystemResult::Ok(ContractResult::from(to_binary(&response)))
            }
        }
        Some(Box::new(A()))
    }

    pub fn create_cw20_failing_info_query_handler() -> Option<Box<dyn HandlesSmartQuery>> {
        struct A();
        impl HandlesSmartQuery for A {
            fn handle(&self, addr: &str, _: &Binary) -> QuerierResult {
                SystemResult::Err(SystemError::NoSuchContract { addr: addr.to_string() })
            }
        }
        Some(Box::new(A()))
    }

    pub fn create_denom_creation_fee_failing_handler() -> Option<Box<dyn HandlesFeeQuery>> {
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

    pub fn create_custom_bank_balance_query_handler(balance: Coin) -> Option<Box<dyn HandlesBankBalanceQuery>> {
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
