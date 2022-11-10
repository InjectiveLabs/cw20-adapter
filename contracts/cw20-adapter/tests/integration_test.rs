#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Uint128, to_binary};
    use cw20::Cw20ExecuteMsg;
    use cw_multi_test::Executor;
    use crate::test_utils::{mock_injective_chain_app, upload_cw20_adapter_contract, upload_cw20_contract};

    pub const CONTRACT_OWNER: &str = "inj1gfawuv6fslzjlfa4v7exv27mk6rpfeyv823eu2";

    #[test]
    fn it_mints_tf_tokens() {
        let owner = Addr::unchecked(CONTRACT_OWNER);
        let mut injective_app = mock_injective_chain_app();
        let cw20_adapter_contract_address = upload_cw20_adapter_contract(&mut injective_app, &owner);

        println!(
            "Adapter address: {:?}",
            cw20_adapter_contract_address
        );

        let cw20_address = upload_cw20_contract(&mut injective_app, &owner);
        println!(
            "cw20 address: {:?}",
            cw20_address
        );

        let mint_msg = Cw20ExecuteMsg::Mint{
            recipient: owner.clone().to_string(),
            amount: Uint128::new(1000),
        };

        let mint_resp = injective_app.execute_contract(owner.clone(), cw20_address.clone(), &mint_msg, &[]).expect("minting of Mieteks failed");
        println!("mint response: {:?}", mint_resp);

        let bin = to_binary(&"").unwrap();

        let send_msg = Cw20ExecuteMsg::Send { contract: cw20_adapter_contract_address.into_string(), amount: Uint128::new(10), msg: bin };

        let send_resp = injective_app.execute_contract(owner, cw20_address, &send_msg, &[]);
                println!("send response: {:?}", send_resp);

        // let ninja_deriv_vault_addr = upload_and_register_ninja_derivative_vault(
        //     &mut injective_app,
        //     &owner,
        //     &master_contract_addr,
        //     deriv_vault_init_msg,
        // );

        // println!(
        //     "Ninja derivative vault address: {:?}",
        //     ninja_deriv_vault_addr
        // );
        // assert!(
        //     &ninja_deriv_vault_addr != &master_contract_addr,
        //     "ninja derivative vault has same address as master address"
        // );

        // let spot_vault_init_msg = mock_spot_init_msg(&owner, &master_contract_addr);

        // let ninja_spot_vault_addr = upload_and_register_ninja_spot_vault(
        //     &mut injective_app,
        //     &owner,
        //     &master_contract_addr,
        //     spot_vault_init_msg,
        // );

        // println!("Ninja spot vault address: {:?}", ninja_spot_vault_addr);
        // assert!(
        //     &ninja_spot_vault_addr != &master_contract_addr,
        //     "ninja spot vault has same address as master address"
        // );

        // assert!(
        //     &ninja_spot_vault_addr != &ninja_deriv_vault_addr,
        //     "ninja spot vault has same address as ninja derivative vault"
        // );

        // let query_registered_msg = QueryMsg::GetRegisteredVaults {
        //     start_after_subaccount: None,
        //     start_after_vault_addr: None,
        //     limit: None,
        // };

        // let registered_vaults: RegisteredVaultsResponse = injective_app
        //     .wrap()
        //     .query_wasm_smart(master_contract_addr.clone(), &query_registered_msg)
        //     .expect("registered vaults query failed");

        // println!("{:?}", registered_vaults);
        // assert_eq!(
        //     registered_vaults.registered_vaults.len(),
        //     2,
        //     "wrong number of registered vaults returned"
        // );

        // assert_first_vault_data(
        //     &ninja_deriv_vault_addr,
        //     &master_contract_addr,
        //     &registered_vaults,
        // );

        // assert_second_vault_data(
        //     &ninja_spot_vault_addr,
        //     &master_contract_addr,
        //     &registered_vaults,
        // );
    }
}

mod test_utils {
    use cosmwasm_std::{testing::MockApi, Addr, Api, Empty, MemoryStorage, StdError, Storage};
    use cw20_adapter::contract::{execute, instantiate, query};
    use cw20_adapter::error::ContractError;
    use cw20_adapter::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use cw20_base::state::MinterData;
    use cw_multi_test::{
        custom_handler::CachingCustomHandler, BankKeeper, BasicAppBuilder, Contract,
        ContractWrapper, DistributionKeeper, Executor, Router, StakeKeeper, WasmKeeper,
    };
    use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
    use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
    use cw20_base::msg::QueryMsg as Cw20QueryMsg;

    use cw20_base::contract::execute as Cw20ExecuteFn;
    use cw20_base::contract::query as Cw20QueryFn;
    use cw20_base::contract::instantiate as Cw20InstantiateFn;

    use cw20_base::ContractError as Cw20ContractError;

    use cw_multi_test::{wasm::AddressGenerator, App};
    use injective_cosmwasm::{
        addr_to_bech32, InjectiveMsgWrapper, InjectiveQueryWrapper, MarketId,
    };
    use std::{fmt::Write, str::FromStr};
    use std::u8;

    use rand::OsRng;
    use secp256k1::Secp256k1;

    const ADDRESS_LENGTH: usize = 40;
    const ADDRESS_BYTES: usize = ADDRESS_LENGTH / 2;
    const KECCAK_OUTPUT_BYTES: usize = 32;
    const ADDRESS_BYTE_INDEX: usize = KECCAK_OUTPUT_BYTES - ADDRESS_BYTES;

    pub const DERIV_MARKET_ID: &str =
        "0x427aee334987c52fa7b567b2662bdbb68614e48c000000000000000000000000";
    pub const SPOT_MARKET_ID: &str =
        "0x01edfab47f124748dc89998eb33144af734484ba07099014594321729a0ca16b";

    type AdapterContractWrapper = ContractWrapper<
        ExecuteMsg,
        InstantiateMsg,
        QueryMsg,
        ContractError,
        StdError,
        StdError,
        InjectiveMsgWrapper,
        Empty,
        Empty,
        anyhow::Error,
        anyhow::Error,
        Empty,
        anyhow::Error,
    >;

    pub fn wrap_cw20_adapter_contract() -> Box<dyn Contract<InjectiveMsgWrapper, Empty>> {
        let contract: AdapterContractWrapper =
            ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    // fn wrapC20ExecuteFn(
    //     deps: DepsMut,
    //     env: Env,
    //     info: MessageInfo,
    //     msg: Cw20ExecuteMsg,) -> Result<Response<InjectiveMsgWrapper>, StdError> {
    //     let result = Cw20ExecuteFn(deps, env, info, msg);

    //     match result {
    //         Ok(r) => {

    //         }
    //         Err(e) => Err(StdError::generic_err( e.to_string()))
    //     }
    // }

    type Cw20ContractWrapper = ContractWrapper<
        Cw20ExecuteMsg,
        Cw20InstantiateMsg,
        Cw20QueryMsg,
        Cw20ContractError,
        Cw20ContractError,
        StdError,
        InjectiveMsgWrapper,
        Empty,
        Empty,
        anyhow::Error,
        anyhow::Error,
        Empty,
        anyhow::Error,
    >;

        pub fn wrap_cw20_contract() -> Box<dyn Contract<InjectiveMsgWrapper, Empty>> {
        let contract: Cw20ContractWrapper =
            ContractWrapper::new(Cw20ExecuteFn, Cw20InstantiateFn, Cw20QueryFn);
        Box::new(contract)
    }

    fn no_init<BankT, CustomT, WasmT, StakingT, DistrT>(
        _: &mut Router<BankT, CustomT, WasmT, StakingT, DistrT>,
        _: &dyn Api,
        _: &mut dyn Storage,
    ) {
    }

    fn to_hex_string(slice: &[u8], expected_string_size: usize) -> String {
        let mut result = String::with_capacity(expected_string_size);

        for &byte in slice {
            write!(&mut result, "{:02x}", byte).expect("Unable to format the public key.");
        }

        result
    }

    pub fn generate_inj_address() -> Addr {
        let secp256k1 = Secp256k1::new();
        let mut rng = OsRng::new().expect("failed to create new random number generator");
        let (_, public_key) = secp256k1
            .generate_keypair(&mut rng)
            .expect("failed to generate key pair");

        let public_key_array = &public_key.serialize_vec(&secp256k1, false)[1..];
        let keccak = tiny_keccak::keccak256(public_key_array);
        let address_short = to_hex_string(&keccak[ADDRESS_BYTE_INDEX..], 40); // get rid of the constant 0x04 byte
        let full_address = format!("0x{}", address_short);
        let inj_address = addr_to_bech32(full_address);

        println!("inj address generated: {:?}", inj_address.clone());

        Addr::unchecked(inj_address)
    }

    struct InjectiveAddressGenerator();

    impl AddressGenerator for InjectiveAddressGenerator {
        fn next_address(&self, _: &mut dyn Storage) -> Addr {
            generate_inj_address()
        }
    }

    type MockedInjectiveApp = App<
        BankKeeper,
        MockApi,
        MemoryStorage,
        CachingCustomHandler<InjectiveMsgWrapper, Empty>,
        WasmKeeper<InjectiveMsgWrapper, Empty>,
        StakeKeeper,
        DistributionKeeper,
    >;

    pub fn mock_injective_chain_app() -> MockedInjectiveApp {
        let custom_handler =
            CachingCustomHandler::<InjectiveMsgWrapper, Empty>::new();

        let inj_wasm_keeper =
            WasmKeeper::<InjectiveMsgWrapper, Empty>::new_with_custom_address_generator(InjectiveAddressGenerator());

        BasicAppBuilder::new()
            .with_custom(custom_handler)
            .with_wasm::<CachingCustomHandler<InjectiveMsgWrapper, Empty>, WasmKeeper<InjectiveMsgWrapper, Empty>>(
                inj_wasm_keeper,
            )
            .build(no_init)
    }

    pub fn upload_cw20_adapter_contract(injective_app: &mut MockedInjectiveApp, owner: &Addr) -> Addr {
        let cw20_adapter_contract_code_id = injective_app.store_code(wrap_cw20_adapter_contract());
        injective_app
            .instantiate_contract(
                cw20_adapter_contract_code_id,
                owner.clone(),
                &InstantiateMsg {},
                &[],
                "cw20 adapter contract",
                None,
            )
            .unwrap()
    }

    pub fn upload_cw20_contract(injective_app: &mut MockedInjectiveApp, owner: &Addr) -> Addr {
        let cw20_contract_code_id = injective_app.store_code(wrap_cw20_contract());

        injective_app
            .instantiate_contract(
                cw20_contract_code_id,
                owner.clone(),
                &Cw20InstantiateMsg {
                    name: "mietek".to_string(),
                    symbol: "MIE".to_string(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(MinterData { minter: owner.clone(), cap: None }),
                    marketing: None,
                },
                &[],
                "cw20 contract",
                Some(owner.to_string()),
            )
            .unwrap()
    }

    // pub fn upload_and_register_ninja_spot_vault(
    //     injective_app: &mut MockedInjectiveApp,
    //     owner: &Addr,
    //     master_contract_addr: &Addr,
    //     instantate_msg: SpotInstantiateMsg,
    // ) -> Addr {
    //     let ninja_spot_contract_code_id = injective_app.store_code(wrap_ninja_spot_contract());

    //     let execute_msg = ExecuteMsg::RegisterVault {
    //         instantiate_vault_msg: ninja_protocol::vault::InstantiateVaultMsg::Spot(instantate_msg),
    //         vault_code_id: ninja_spot_contract_code_id,
    //         vault_label: "ninja spot vault".to_string(),
    //     };

    //     let resp = injective_app
    //         .execute_contract(
    //             owner.clone(),
    //             master_contract_addr.clone(),
    //             &execute_msg,
    //             &[],
    //         )
    //         .expect("ninja spot registration failed");

    //     let ninja_address = resp
    //         .events
    //         .iter()
    //         .find(|e| e.ty == "instantiate")
    //         .expect("ninja spot contract was't initialised")
    //         .attributes
    //         .iter()
    //         .find(|a| a.key == "_contract_addr")
    //         .expect("no attribute with contract address found")
    //         .value
    //         .clone();

    //     Addr::unchecked(ninja_address)
    // }
}
