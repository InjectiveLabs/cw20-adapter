use cosmwasm_std::{Addr, Coin, QuerierWrapper, StdResult};
use injective_cosmwasm::{InjectiveQuerier, InjectiveQueryWrapper};
use regex::Regex;

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

pub mod test_utils {
    use cosmwasm_std::{Addr, Binary, BlockInfo, ContractInfo, ContractResult, Env, QuerierResult, SystemError, SystemResult, Timestamp, to_binary, TransactionInfo, Uint128};
    use cw20::TokenInfoResponse;
    use injective_cosmwasm::HandlesSmartQuery;

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
                    name: "lp token".to_string(),
                    symbol: "LPT".to_string(),
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
}
