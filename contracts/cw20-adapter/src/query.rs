use cosmwasm_std::{Coin, Deps, Order, StdResult};

use crate::state::CW20_CONTRACTS;

pub fn registered_contracts(deps: Deps) -> StdResult<Vec<String>> {
    let contracts = CW20_CONTRACTS
        .items(deps.storage, None, None, Order::Ascending)
        .filter_map(|c| c.ok())
        .collect();
    Ok(contracts)
}

pub fn new_denom_fee(deps: Deps) -> StdResult<Coin> {
    // TODO: implement real query
    Ok(Coin::new(10, "inj"))
}
