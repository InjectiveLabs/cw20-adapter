use cosmwasm_schema::schemars::JsonSchema;
use cosmwasm_schema::serde::{Deserialize, Serialize};
use cosmwasm_std::Addr;
use cw_item_set::Set;
use cw_storage_plus::{Index, IndexedMap, IndexList, Item, Map, MultiIndex, UniqueIndex};


pub const CW20_CONTRACTS: Set<&str> = Set::new("contracts", "contracts__counter");

