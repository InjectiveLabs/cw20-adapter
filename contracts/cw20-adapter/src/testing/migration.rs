use crate::{
    contract::{migrate, CONTRACT_NAME, CONTRACT_VERSION},
    msg::MigrateMsg,
    state::CW20_CONTRACTS,
    testing::utils::{
        instantiate, instantiate_cw20, mint_cw20, query_balance_cw20, query_register_contract, redeem_and_transfer, register_contract, send_cw20,
        store_code_with_path,
    },
};

use cosmwasm_std::{coins, from_json, testing::mock_env, Storage, Uint128};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use injective_cosmwasm::mock_dependencies;
use injective_std::types::cosmwasm::wasm::v1::{QueryRawContractStateRequest, QueryRawContractStateResponse};
use injective_test_tube::{Account, InjectiveTestApp, Module, Runner, Wasm};
use injective_testing::test_tube::utils::store_code;
use testenv::utils::Setup;

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

fn query_contract_version(app: &InjectiveTestApp, contract: &str) -> ContractVersion {
    let response = app
        .query::<QueryRawContractStateRequest, QueryRawContractStateResponse>(
            "/cosmwasm.wasm.v1.Query/RawContractState",
            &QueryRawContractStateRequest {
                address: contract.to_string(),
                query_data: b"contract_info".to_vec(),
            },
        )
        .unwrap();

    from_json(response.data).unwrap()
}

#[test]
fn test_tube_migrates_deployed_v1_and_preserves_state() {
    let env = Setup::new();
    let wasm = Wasm::new(&env.app);

    // This fixture is the exact Wasm stored as code ID 8 on injective-1.
    let legacy_code_id = store_code_with_path(
        &wasm,
        &env.owner,
        "../../contracts/cw20-adapter/src/testing/test_artifacts/cw20_adapter_v1.wasm".to_string(),
    );
    let new_code_id = store_code(&wasm, &env.owner, "cw20_adapter".to_string());
    let cw20_code_id = store_code_with_path(
        &wasm,
        &env.owner,
        "../../contracts/cw20-adapter/src/testing/test_artifacts/cw20_base.wasm".to_string(),
    );

    let adapter_addr = instantiate(&env.app, &env.owner, legacy_code_id, "legacy-cw20-adapter");
    let cw20_addr = instantiate_cw20(&env.app, &env.owner, cw20_code_id, "cw20-base", "migration-token");
    let canonical_denom = format!("factory/{adapter_addr}/{cw20_addr}");

    register_contract(
        &env.app,
        &env.owner,
        &adapter_addr,
        &cw20_addr,
        coins(10_000_000_000_000_000_000u128, "inj"),
    )
    .unwrap();

    let reserve_amount = Uint128::new(2_000_000);
    let trader = &env.traders[0].account;
    mint_cw20(&env.app, &env.owner, &cw20_addr, reserve_amount, trader.address()).unwrap();
    send_cw20(&env.app, trader, &cw20_addr, adapter_addr.clone(), reserve_amount).unwrap();

    assert_eq!(
        query_contract_version(&env.app, &adapter_addr),
        ContractVersion {
            contract: CONTRACT_NAME.to_string(),
            version: "1.0.0".to_string(),
        }
    );
    assert_eq!(query_register_contract(&env.app, &adapter_addr).unwrap(), vec![cw20_addr.clone()]);
    assert_eq!(
        query_balance_cw20(&env.app, &cw20_addr, adapter_addr.clone()).unwrap().balance,
        reserve_amount
    );
    assert_eq!(env.get_balance(trader.address(), canonical_denom.clone()), reserve_amount);

    wasm.migrate(new_code_id, &adapter_addr, &MigrateMsg {}, &env.owner).unwrap();

    assert_eq!(
        query_contract_version(&env.app, &adapter_addr),
        ContractVersion {
            contract: CONTRACT_NAME.to_string(),
            version: CONTRACT_VERSION.to_string(),
        }
    );
    assert_eq!(query_register_contract(&env.app, &adapter_addr).unwrap(), vec![cw20_addr.clone()]);
    assert_eq!(
        query_balance_cw20(&env.app, &cw20_addr, adapter_addr.clone()).unwrap().balance,
        reserve_amount
    );

    redeem_and_transfer(
        &env.app,
        trader,
        &adapter_addr,
        None,
        coins(reserve_amount.u128(), canonical_denom.clone()),
    )
    .unwrap();

    assert!(env.get_balance(trader.address(), canonical_denom).is_zero());
    assert_eq!(
        query_balance_cw20(&env.app, &cw20_addr, trader.address()).unwrap().balance,
        reserve_amount
    );
    assert!(query_balance_cw20(&env.app, &cw20_addr, adapter_addr).unwrap().balance.is_zero());
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
