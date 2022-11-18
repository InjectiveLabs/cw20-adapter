#![allow(dead_code)]

use cosmwasm_std::{
    to_binary, Addr, BalanceResponse, Binary, BlockInfo, Coin, ContractInfo, ContractResult, Env, QuerierResult, SystemError, SystemResult,
    Timestamp, TransactionInfo, Uint128,
};
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
