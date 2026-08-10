use crate::{
    error::ContractError,
    testing::utils::{
        assert_execute_error, instantiate, instantiate_cw20, mint_cw20, redeem_and_transfer, register_contract, send_cw20, store_code_with_path,
    },
};

use cosmwasm_std::{coins, Uint128};
use injective_std::types::{
    cosmos::base::v1beta1::Coin as ProtoCoin,
    injective::tokenfactory::v1beta1::{MsgCreateDenom, MsgMint},
};
use injective_test_tube::{Account, Module, TokenFactory, Wasm};
use injective_testing::test_tube::utils::store_code;
use testenv::utils::Setup;

use super::utils::query_balance_cw20;

#[test]
fn test_redeem() {
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

    send_cw20(&env.app, &env.traders[0].account, &cw20_addr, adaptor_addr.clone(), mint_amount).unwrap();

    let funds = coins(mint_amount.into(), format!("factory/{}/{}", adaptor_addr, cw20_addr.clone()));
    redeem_and_transfer(&env.app, &env.traders[0].account, &adaptor_addr, None, funds).unwrap();

    let trader_balance = env.get_balance(
        env.traders[0].account.address(),
        format!("factory/{}/{}", adaptor_addr, cw20_addr.clone()),
    );
    assert!(trader_balance.is_zero());

    let recipient_balance = query_balance_cw20(&env.app, &cw20_addr.clone(), env.traders[0].account.address()).unwrap();
    assert_eq!(recipient_balance.balance, mint_amount);
}

#[test]
fn test_redeem_and_transfer() {
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

    send_cw20(&env.app, &env.traders[0].account, &cw20_addr, adaptor_addr.clone(), mint_amount).unwrap();

    let funds = coins(mint_amount.into(), format!("factory/{}/{}", adaptor_addr, cw20_addr.clone()));
    redeem_and_transfer(
        &env.app,
        &env.traders[0].account,
        &adaptor_addr,
        Some(env.traders[6].account.address()),
        funds,
    )
    .unwrap();

    let trader_balance = env.get_balance(
        env.traders[0].account.address(),
        format!("factory/{}/{}", adaptor_addr, cw20_addr.clone()),
    );
    assert!(trader_balance.is_zero());

    let recipient_balance = query_balance_cw20(&env.app, &cw20_addr.clone(), env.traders[6].account.address()).unwrap();
    assert_eq!(recipient_balance.balance, mint_amount);
}

#[test]
fn test_redeem_rejects_lookalike_denom_owned_by_attacker() {
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

    let attack_amount = Uint128::from(2_000_000u128);
    mint_cw20(&env.app, &env.owner, &cw20_addr, attack_amount, adaptor_addr.clone()).unwrap();

    let attacker = &env.traders[0].account;
    let token_factory = TokenFactory::new(&env.app);
    let forged_denom = token_factory
        .create_denom(
            MsgCreateDenom {
                sender: attacker.address(),
                subdenom: cw20_addr.clone(),
                name: "Forged adapter token".to_string(),
                symbol: "FORGED".to_string(),
                decimals: 6,
            },
            attacker,
        )
        .unwrap()
        .data
        .new_token_denom;

    token_factory
        .mint(
            MsgMint {
                sender: attacker.address(),
                amount: Some(ProtoCoin {
                    denom: forged_denom.clone(),
                    amount: attack_amount.to_string(),
                }),
            },
            attacker,
        )
        .unwrap();

    let reserve_before = query_balance_cw20(&env.app, &cw20_addr, adaptor_addr.clone()).unwrap();
    let forged_balance_before = env.get_balance(attacker.address(), forged_denom.clone());

    let err = redeem_and_transfer(
        &env.app,
        attacker,
        &adaptor_addr,
        Some(attacker.address()),
        coins(attack_amount.u128(), forged_denom.clone()),
    )
    .unwrap_err();

    assert_eq!(err.to_string(), assert_execute_error(&ContractError::InvalidAdapterDenom.to_string()));

    let reserve_after = query_balance_cw20(&env.app, &cw20_addr, adaptor_addr).unwrap();
    let forged_balance_after = env.get_balance(attacker.address(), forged_denom);

    assert_eq!(reserve_after, reserve_before);
    assert_eq!(forged_balance_after, forged_balance_before);
}
