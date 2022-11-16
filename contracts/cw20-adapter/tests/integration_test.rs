#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_info;
    use cosmwasm_std::{Addr, Coin, Uint128};
    use cw20_adapter::common::get_denom;
    use injective_cosmwasm::{
        create_simple_balance_bank_query_handler, mock_dependencies, WasmMockQuerier,
    };

    use crate::test_utils::mock_env;
    use cw20_adapter::contract::{execute, instantiate};
    use cw20_adapter::msg::{ExecuteMsg, InstantiateMsg};

    pub const ADAPTER_CONTRACT: &str = "inj1zwv6feuzhy6a9wekh96cd57lsarmqlwxvdl4nk";
    pub const CW20_CONTRACT: &str = "inj1h0y3hssxf4vsdacfmjg720642cvpxwyqh35kpn";
    pub const ADMIN: &str = "inj1qg5ega6dykkxc307y25pecuufrjkxkag6xhp6y";
    pub const USER: &str = "inj1gfawuv6fslzjlfa4v7exv27mk6rpfeyv823eu2";

    #[test]
    fn it_mints_tf_tokens() {
        let admin = Addr::unchecked(ADMIN);

        let mut deps = mock_dependencies();
        let mut wasm_querier = WasmMockQuerier::new();

        wasm_querier.balance_query_handler =
            create_simple_balance_bank_query_handler(vec![Coin::new(10, "inj")]);
        // wasm_querier.spot_market_response_handler = create_spot_market_handler(get_market(MarketId::unchecked("normal")));
        // wasm_querier.smart_query_handler = create_smart_query_handler();
        deps.querier = wasm_querier;

        let msg = InstantiateMsg {};

        let info_inst = mock_info(ADMIN, &[]);
        // we can just call .unwrap() to assert this was a success
        let res_inst =
            instantiate(deps.as_mut(), mock_env(ADAPTER_CONTRACT), info_inst, msg).unwrap();

        // send some tokens to a contract
        let info_receive = mock_info(CW20_CONTRACT, &[]);
        let msg = ExecuteMsg::Receive {
            sender: USER.to_string(),
            amount: Uint128::new(1000),
            msg: Default::default(),
        };
        let res_receive =
            execute(deps.as_mut(), mock_env(ADAPTER_CONTRACT), info_receive, msg).unwrap();

        let denom = get_denom(&Addr::unchecked(ADAPTER_CONTRACT), &Addr::unchecked(CW20_CONTRACT));
        // redeem some tokens to a contract
        let info_redeem = mock_info(ADAPTER_CONTRACT, &[Coin::new(800, denom)]);
        let msg = ExecuteMsg::Redeem {
            recipient: USER.to_string(),
        };
        let res_redeem = execute(deps.as_mut(), mock_env(ADAPTER_CONTRACT), info_redeem, msg);

        assert!(res_redeem.is_ok())
        // // it worked, let's query the state
        // let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        // let config: ConfigResponse = from_binary(&res).unwrap();
        //
        // assert_eq!(
        //     ConfigResponse {
        //         owner: "owner".to_string(),
        //         ninja_token: "reward".to_string(),
        //         distribution_contract: "distribution".to_string(),
        //     },
        //     config
        // );
        // let mint_msg = Cw20ExecuteMsg::Mint {
        //     recipient: owner.clone().to_string(),
        //     amount: Uint128::new(1000),
        // };

        // let mint_resp = injective_app.execute_contract(owner.clone(), cw20_address.clone(), &mint_msg, &[]).expect("minting of Mieteks failed");
        // println!("mint response: {:?}", mint_resp);
        //
        // let bin = to_binary(&"").unwrap();
        //
        // let send_msg = Cw20ExecuteMsg::Send { contract: cw20_adapter_contract_address.into_string(), amount: Uint128::new(10), msg: bin };
        //
        // let send_resp = injective_app.execute_contract(owner, cw20_address, &send_msg, &[]);
        // println!("send response: {:?}", send_resp);
    }
}

mod test_utils {
    use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
    use cosmwasm_std::{Addr, BlockInfo, ContractInfo, Env, Timestamp, TransactionInfo};

    pub fn mock_env(addr: &str) -> Env {
        Env {
            block: BlockInfo {
                height: 12_345,
                time: Timestamp::from_nanos(1_571_797_419_879_305_533),
                chain_id: "inj-testnet-14002".to_string(),
            },
            transaction: Some(TransactionInfo {
                index: 3,
            }),
            contract: ContractInfo {
                address: Addr::unchecked(addr),
            },
        }
    }

    //     type AdapterContractWrapper = ContractWrapper<
    //         ExecuteMsg, // execute_fn
    //         InstantiateMsg, // instantiate_fn
    //         QueryMsg, // query_fn
    //         ContractError, // execute_fn
    //         StdError, // execute_fn
    //         StdError, // execute_fns
    //         InjectiveMsgWrapper, // Result<Response<C>, E>>
    //         InjectiveQueryWrapper, // DepsMut<Q>
    //         Empty,
    //         anyhow::Error,
    //         anyhow::Error,
    //         Empty,
    //         anyhow::Error,
    //     >;
    //
    //     pub fn wrap_cw20_adapter_contract() -> Box<dyn Contract<InjectiveMsgWrapper, InjectiveQueryWrapper>> {
    //         let contract: AdapterContractWrapper =
    //             ContractWrapper::new(execute, instantiate, query);
    //         Box::new(contract)
    //     }
    //
    //     // fn wrapC20ExecuteFn(
    //     //     deps: DepsMut,
    //     //     env: Env,
    //     //     info: MessageInfo,
    //     //     msg: Cw20ExecuteMsg,) -> Result<Response<InjectiveMsgWrapper>, StdError> {
    //     //     let result = Cw20ExecuteFn(deps, env, info, msg);
    //
    //     //     match result {
    //     //         Ok(r) => {
    //
    //     //         }
    //     //         Err(e) => Err(StdError::generic_err( e.to_string()))
    //     //     }
    //     // }
    //
    //     type Cw20ContractWrapper = ContractWrapper<
    //         Cw20ExecuteMsg,
    //         Cw20InstantiateMsg,
    //         Cw20QueryMsg,
    //         Cw20ContractError,
    //         Cw20ContractError,
    //         StdError,
    //         InjectiveMsgWrapper,
    //         InjectiveQueryWrapper,
    //         Empty,
    //         anyhow::Error,
    //         anyhow::Error,
    //         Empty,
    //         anyhow::Error,
    //     >;
    //
    //         pub fn wrap_cw20_contract() -> Box<dyn Contract<InjectiveMsgWrapper, InjectiveQueryWrapper>> {
    //         let contract: Cw20ContractWrapper =
    //             ContractWrapper::new(Cw20ExecuteFn, Cw20InstantiateFn, Cw20QueryFn);
    //         Box::new(contract)
    //     }
    //
    //     fn no_init<BankT, CustomT, WasmT, StakingT, DistrT>(
    //         _: &mut Router<BankT, CustomT, WasmT, StakingT, DistrT>,
    //         _: &dyn Api,
    //         _: &mut dyn Storage,
    //     ) {
    //     }
    //
    //     fn to_hex_string(slice: &[u8], expected_string_size: usize) -> String {
    //         let mut result = String::with_capacity(expected_string_size);
    //
    //         for &byte in slice {
    //             write!(&mut result, "{:02x}", byte).expect("Unable to format the public key.");
    //         }
    //
    //         result
    //     }
    //
    //     pub fn generate_inj_address() -> Addr {
    //         let secp256k1 = Secp256k1::new();
    //         let mut rng = OsRng::new().expect("failed to create new random number generator");
    //         let (_, public_key) = secp256k1
    //             .generate_keypair(&mut rng)
    //             .expect("failed to generate key pair");
    //
    //         let public_key_array = &public_key.serialize_vec(&secp256k1, false)[1..];
    //         let keccak = tiny_keccak::keccak256(public_key_array);
    //         let address_short = to_hex_string(&keccak[ADDRESS_BYTE_INDEX..], 40); // get rid of the constant 0x04 byte
    //         let full_address = format!("0x{}", address_short);
    //         let inj_address = addr_to_bech32(full_address);
    //
    //         println!("inj address generated: {:?}", inj_address.clone());
    //
    //         Addr::unchecked(inj_address)
    //     }
    //
    //     struct InjectiveAddressGenerator();
    //
    //     impl AddressGenerator for InjectiveAddressGenerator {
    //         fn next_address(&self, _: &mut dyn Storage) -> Addr {
    //             generate_inj_address()
    //         }
    //     }
    //
    //     type MockedInjectiveApp = App<
    //         BankKeeper,
    //         MockApi,
    //         MemoryStorage,
    //         CachingCustomHandler<InjectiveMsgWrapper, InjectiveQueryWrapper>,
    //         WasmKeeper<InjectiveMsgWrapper, InjectiveQueryWrapper>,
    //         StakeKeeper,
    //         DistributionKeeper,
    //     >;
    //
    //     pub fn mock_injective_chain_app() -> MockedInjectiveApp {
    //         let custom_handler =
    //             CachingCustomHandler::<InjectiveMsgWrapper, InjectiveQueryWrapper>::new();
    //
    //         let inj_wasm_keeper =
    //             WasmKeeper::<InjectiveMsgWrapper, InjectiveQueryWrapper>::new_with_custom_address_generator(InjectiveAddressGenerator());
    //
    //         BasicAppBuilder::new()
    //             .with_custom(custom_handler)
    //             .with_wasm::<CachingCustomHandler<InjectiveMsgWrapper, InjectiveQueryWrapper>, WasmKeeper<InjectiveMsgWrapper, InjectiveQueryWrapper>>(
    //                 inj_wasm_keeper,
    //             )
    //             .build(no_init)
    //     }
    //
    //     pub fn upload_cw20_adapter_contract(injective_app: &mut MockedInjectiveApp, owner: &Addr) -> Addr {
    //         let cw20_adapter_contract_code_id = injective_app.store_code(wrap_cw20_adapter_contract());
    //         injective_app
    //             .instantiate_contract(
    //                 cw20_adapter_contract_code_id,
    //                 owner.clone(),
    //                 &InstantiateMsg {},
    //                 &[],
    //                 "cw20 adapter contract",
    //                 None,
    //             )
    //             .unwrap()
    //     }
    //
    //     pub fn upload_cw20_contract(injective_app: &mut MockedInjectiveApp, owner: &Addr) -> Addr {
    //         let cw20_contract_code_id = injective_app.store_code(wrap_cw20_contract());
    //
    //         injective_app
    //             .instantiate_contract(
    //                 cw20_contract_code_id,
    //                 owner.clone(),
    //                 &Cw20InstantiateMsg {
    //                     name: "mietek".to_string(),
    //                     symbol: "MIE".to_string(),
    //                     decimals: 6,
    //                     initial_balances: vec![],
    //                     mint: Some(MinterData { minter: owner.clone(), cap: None }),
    //                     marketing: None,
    //                 },
    //                 &[],
    //                 "cw20 contract",
    //                 Some(owner.to_string()),
    //             )
    //             .unwrap()
    //     }
    //
    //     // pub fn upload_and_register_ninja_spot_vault(
    //     //     injective_app: &mut MockedInjectiveApp,
    //     //     owner: &Addr,
    //     //     master_contract_addr: &Addr,
    //     //     instantate_msg: SpotInstantiateMsg,
    //     // ) -> Addr {
    //     //     let ninja_spot_contract_code_id = injective_app.store_code(wrap_ninja_spot_contract());
    //
    //     //     let execute_msg = ExecuteMsg::RegisterVault {
    //     //         instantiate_vault_msg: ninja_protocol::vault::InstantiateVaultMsg::Spot(instantate_msg),
    //     //         vault_code_id: ninja_spot_contract_code_id,
    //     //         vault_label: "ninja spot vault".to_string(),
    //     //     };
    //
    //     //     let resp = injective_app
    //     //         .execute_contract(
    //     //             owner.clone(),
    //     //             master_contract_addr.clone(),
    //     //             &execute_msg,
    //     //             &[],
    //     //         )
    //     //         .expect("ninja spot registration failed");
    //
    //     //     let ninja_address = resp
    //     //         .events
    //     //         .iter()
    //     //         .find(|e| e.ty == "instantiate")
    //     //         .expect("ninja spot contract was't initialised")
    //     //         .attributes
    //     //         .iter()
    //     //         .find(|a| a.key == "_contract_addr")
    //     //         .expect("no attribute with contract address found")
    //     //         .value
    //     //         .clone();
    //
    //     //     Addr::unchecked(ninja_address)
    //     // }
}
