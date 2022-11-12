use cosmwasm_std::{Coin, Deps, Order, StdResult};
use injective_cosmwasm::{InjectiveQuerier, InjectiveQueryWrapper};

use crate::state::CW20_CONTRACTS;

pub fn registered_contracts(deps: Deps<InjectiveQueryWrapper>) -> StdResult<Vec<String>> {
    let contracts = CW20_CONTRACTS
        .items(deps.storage, None, None, Order::Ascending)
        .filter_map(|c| c.ok())
        .collect();
    Ok(contracts)
}

pub fn new_denom_fee(deps: Deps<InjectiveQueryWrapper>) -> StdResult<Vec<Coin>> {
    let querier = InjectiveQuerier::new(&deps.querier);
    Ok(querier.query_token_factory_creation_fee()?.fee)
}
