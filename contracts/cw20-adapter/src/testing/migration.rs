use crate::{
    contract::{migrate, CONTRACT_NAME, CONTRACT_VERSION},
    msg::MigrateMsg,
    state::CW20_CONTRACTS,
};

use cosmwasm_std::{testing::mock_env, Storage};
use cw2::{get_contract_version, set_contract_version};
use injective_cosmwasm::mock_dependencies;

const REGISTERED_CW20: &str = "inj1pjcw9hhx8kf462qtgu37p7l7shyqgpfr82r6em";

fn legacy_set_key(item: &str) -> Vec<u8> {
    let namespace = b"contracts";
    let length = (namespace.len() as u32).to_be_bytes();
    let mut key = Vec::with_capacity(2 + namespace.len() + item.len());
    key.extend_from_slice(&length[2..]);
    key.extend_from_slice(namespace);
    key.extend_from_slice(item.as_bytes());
    key
}

#[test]
fn test_migration_updates_version_and_preserves_registered_contracts() {
    let mut deps = mock_dependencies();
    set_contract_version(&mut deps.storage, CONTRACT_NAME, "1.0.0").unwrap();

    // Reproduce the cw-item-set 0.6 storage layout used by the deployed v1 contract.
    deps.storage.set(&legacy_set_key(REGISTERED_CW20), b"{}");
    deps.storage.set(b"contracts__counter", b"1");

    let response = migrate(deps.as_mut(), mock_env(), MigrateMsg {}).unwrap();

    assert_eq!(response.attributes[0].value, "migrate");
    assert_eq!(response.attributes[1].value, "1.0.0");
    assert_eq!(response.attributes[2].value, CONTRACT_VERSION);
    assert!(CW20_CONTRACTS.contains(&deps.storage, REGISTERED_CW20));
    assert_eq!(CW20_CONTRACTS.count(&deps.storage).unwrap(), 1);

    let stored_version = get_contract_version(&deps.storage).unwrap();
    assert_eq!(stored_version.contract, CONTRACT_NAME);
    assert_eq!(stored_version.version, CONTRACT_VERSION);
}

#[test]
fn test_migration_rejects_unrelated_contract_state() {
    let mut deps = mock_dependencies();
    set_contract_version(&mut deps.storage, "unrelated-contract", "1.0.0").unwrap();

    let err = migrate(deps.as_mut(), mock_env(), MigrateMsg {}).unwrap_err();

    assert!(err.to_string().contains("Cannot migrate from unrelated-contract"));
}
