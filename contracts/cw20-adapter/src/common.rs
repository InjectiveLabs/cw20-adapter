use cosmwasm_std::{Addr, Coin, CosmosMsg, DepsMut, Env, QuerierWrapper, StdResult, Uint128};
use cw20::{Cw20QueryMsg, TokenInfoResponse};

use injective_cosmwasm::{create_new_denom_msg, InjectiveMsgWrapper, InjectiveQuerier, InjectiveQueryWrapper};
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::state::CW20_CONTRACTS;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdapterDenom {
    pub adapter_addr: String,
    pub cw20_addr: String,
}

impl AdapterDenom {
    pub fn new<S>(denom: S) -> Result<Self, ContractError>
    where
        S: Into<String>,
    {
        let denom = denom.into();

        if denom.len() != 93 {
            return Err(ContractError::NotCw20Address);
        }
        if !denom.starts_with("factory/") {
            return Err(ContractError::NotCw20Address);
        }
        let adapter_part = &denom[8..50];
        let cw20_part = &denom[51..];
        Ok::<Result<AdapterDenom, ContractError>, ContractError>(AdapterDenom::from_components(adapter_part, cw20_part))?
    }

    pub fn from_components<S>(adapter_addr: S, cw20_addr: S) -> Result<Self, ContractError>
    where
        S: Into<String>,
    {
        let adapter_addr = adapter_addr.into();
        let cw20_addr = cw20_addr.into();
        if !adapter_addr.chars().all(char::is_alphanumeric) {
            return Err(ContractError::NotCw20Address);
        }
        if !cw20_addr.chars().all(char::is_alphanumeric) {
            return Err(ContractError::NotCw20Address);
        }
        Ok(AdapterDenom { adapter_addr, cw20_addr })
    }

    pub fn as_string(&self) -> String {
        get_denom_from_str(&self.adapter_addr, &self.cw20_addr)
    }
}

pub struct AdapterCoin {
    pub amount: Uint128,
    pub denom: AdapterDenom,
}

impl AdapterCoin {
    pub fn as_coin(&self) -> Coin {
        Coin::new(self.amount.u128(), self.denom.as_string())
    }
}

pub fn get_denom(adapter_address: &Addr, cw20addr: &Addr) -> String {
    get_denom_from_str(adapter_address.as_str(), cw20addr.as_str())
}

fn get_denom_from_str(adapter_address: &str, cw20addr: &str) -> String {
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
        AdapterDenom::new("factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h")
            .expect("input was not treated as token factory denom");
    }

    #[test]
    fn it_returns_false_for_non_token_factory_denom() {
        AdapterDenom::new(".factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7")
            .expect_err("input was treated as token factory denom");
    }

    #[test]
    fn it_returns_false_for_non_token_factory_denom_2() {
        AdapterDenom::new("factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtz/winj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h")
            .expect_err("input was treated as token factory denom");
    }

    #[test]
    fn it_returns_false_for_non_token_factory_denom_3() {
        AdapterDenom::new("factory/inj1pvrwm_uusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7")
            .expect_err("input was treated as token factory denom");
    }

    #[test]
    fn it_returns_false_for_non_token_factory_denom_4() {
        AdapterDenom::new("factory/inj1pvrwmuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7/sddsjsdk")
            .expect_err("input was treated as token factory denom");
    }

    #[test]
    fn it_returns_cw_20_address_for_token_factory_denom() {
        let denom = AdapterDenom::new("factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h").unwrap();
        assert_eq!(
            denom.adapter_addr, "inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw",
            "wrong cw20 address returned"
        );
        assert_eq!(
            denom.cw20_addr, "inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h",
            "wrong cw20 address returned"
        )
    }

    #[test]
    fn it_returns_none_cw_20_address_for_non_token_factory_denom() {
        let denom = AdapterDenom::new("factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7").unwrap_err();
        assert_eq!(denom, ContractError::NotCw20Address, "cw20 address returned")
    }

    #[test]
    fn it_returns_denom() {
        let denom =
            AdapterDenom::from_components("inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw", "inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h").unwrap();
        assert_eq!(
            denom.as_string(),
            "factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h",
            "wrong denom returned"
        )
    }
}
