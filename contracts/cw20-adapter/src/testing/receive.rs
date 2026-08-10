use crate::{
    error::ContractError,
    testing::utils::{
        assert_execute_error, instantiate, instantiate_cw20, mint_cw20, query_register_contract, register_contract, send_cw20, store_code_with_path,
    },
};

use cosmwasm_std::{coins, Uint128};
use injective_std::types::cosmos::{bank::v1beta1::MsgSend, base::v1beta1::Coin as ProtoCoin};
use injective_test_tube::{Account, Bank, Module, Wasm};
use injective_testing::test_tube::utils::store_code;
use testenv::utils::Setup;

use super::utils::query_balance_cw20;

#[test]
fn test_receive() {
    let env = Setup::new();

    let wasm = Wasm::new(&env.app);

    let adaptor_code_id = store_code(&wasm, &env.owner, "cw20_adapter".to_string());
    let base_code_id = store_code_with_path(
        &wasm,
        &env.owner,
        "../../contracts/cw20-adapter/src/testing/test_artifacts/cw20_base.wasm".to_string(),
    );

    let adaptor_addr = instantiate(&env.app, &env.owner, adaptor_code_id, "cw20-adapter");
    let cw20_addr = instantiate_cw20(&env.app, &env.owner, base_code_id, "cw20-base", "cw20-mofo");

    let deposit = coins(10_000_000_000_000_000_000u128, "inj");
    register_contract(&env.app, &env.owner, &adaptor_addr, &cw20_addr, deposit).unwrap();

    let mint_amount = Uint128::from(2_000_000u128);
    mint_cw20(&env.app, &env.owner, &cw20_addr, mint_amount, env.traders[0].account.address()).unwrap();

    let adaptor_balance_before = query_balance_cw20(&env.app, &cw20_addr, adaptor_addr.clone()).unwrap();
    assert!(adaptor_balance_before.balance.is_zero());

    send_cw20(&env.app, &env.traders[0].account, &cw20_addr, adaptor_addr.clone(), mint_amount).unwrap();

    let adaptor_balance_after = query_balance_cw20(&env.app, &cw20_addr, adaptor_addr.clone()).unwrap();
    assert_eq!(adaptor_balance_after.balance, mint_amount);

    let trader_balance = env.get_balance(env.traders[0].account.address(), format!("factory/{}/{}", adaptor_addr, cw20_addr));
    assert_eq!(trader_balance, mint_amount);
}

#[test]
fn test_receive_rejects_unregistered_cw20_without_spending_adapter_funds() {
    let env = Setup::new();

    let wasm = Wasm::new(&env.app);

    let adaptor_code_id = store_code(&wasm, &env.owner, "cw20_adapter".to_string());
    let base_code_id = store_code_with_path(
        &wasm,
        &env.owner,
        "../../contracts/cw20-adapter/src/testing/test_artifacts/cw20_base.wasm".to_string(),
    );

    let adaptor_addr = instantiate(&env.app, &env.owner, adaptor_code_id, "cw20-adapter");
    let cw20_addr = instantiate_cw20(&env.app, &env.owner, base_code_id, "cw20-base", "cw20-mofo");

    let denom_creation_fee = 10_000_000_000_000_000_000u128;
    Bank::new(&env.app)
        .send(
            MsgSend {
                from_address: env.owner.address(),
                to_address: adaptor_addr.clone(),
                amount: vec![ProtoCoin {
                    denom: "inj".to_string(),
                    amount: denom_creation_fee.to_string(),
                }],
            },
            &env.owner,
        )
        .unwrap();

    let mint_amount = Uint128::from(2_000_000u128);
    mint_cw20(&env.app, &env.owner, &cw20_addr, mint_amount, env.traders[0].account.address()).unwrap();

    let adapter_cw20_balance_before = query_balance_cw20(&env.app, &cw20_addr, adaptor_addr.clone()).unwrap();
    let trader_cw20_balance_before = query_balance_cw20(&env.app, &cw20_addr, env.traders[0].account.address()).unwrap();
    let adapter_inj_balance_before = env.get_balance(adaptor_addr.clone(), "inj".to_string());
    assert!(adapter_cw20_balance_before.balance.is_zero());
    assert_eq!(trader_cw20_balance_before.balance, mint_amount);
    assert_eq!(adapter_inj_balance_before, Uint128::new(denom_creation_fee));

    let err = send_cw20(&env.app, &env.traders[0].account, &cw20_addr, adaptor_addr.clone(), mint_amount).unwrap_err();

    assert_eq!(
        err.to_string(),
        assert_execute_error(&format!("dispatch: submessages: {}", ContractError::ContractNotRegistered))
    );

    let adapter_cw20_balance_after = query_balance_cw20(&env.app, &cw20_addr, adaptor_addr.clone()).unwrap();
    let trader_cw20_balance_after = query_balance_cw20(&env.app, &cw20_addr, env.traders[0].account.address()).unwrap();
    let adapter_inj_balance_after = env.get_balance(adaptor_addr.clone(), "inj".to_string());
    let registered_contracts = query_register_contract(&env.app, &adaptor_addr).unwrap();

    assert!(adapter_cw20_balance_after.balance.is_zero());
    assert_eq!(trader_cw20_balance_after.balance, mint_amount);
    assert_eq!(adapter_inj_balance_after, adapter_inj_balance_before);
    assert!(registered_contracts.is_empty());
}
