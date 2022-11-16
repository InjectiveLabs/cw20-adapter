use cosmwasm_std::{Coin, Deps, Order, StdResult};
use injective_cosmwasm::InjectiveQueryWrapper;
use crate::common::query_denom_creation_fee;

use crate::state::CW20_CONTRACTS;

pub fn registered_contracts(deps: Deps<InjectiveQueryWrapper>) -> StdResult<Vec<String>> {
    let contracts = CW20_CONTRACTS
        .items(deps.storage, None, None, Order::Ascending)
        .filter_map(|c| c.ok())
        .collect();
    Ok(contracts)
}

pub fn new_denom_fee(deps: Deps<InjectiveQueryWrapper>) -> StdResult<Vec<Coin>> {
    query_denom_creation_fee(&deps.querier)
}
