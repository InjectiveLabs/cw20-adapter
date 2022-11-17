#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_info;
    use cosmwasm_std::{to_binary, Addr, Coin, Uint128};
    use cw20_adapter::common::get_denom;
    use injective_cosmwasm::{create_simple_balance_bank_query_handler, create_smart_query_handler, mock_dependencies, WasmMockQuerier};
    use cw20_adapter::common::test_utils::{create_cw20_info_query_handler, mock_env};

    use cw20_adapter::contract::{execute, instantiate};
    use cw20_adapter::msg::{ExecuteMsg, InstantiateMsg};

    pub const ADAPTER_CONTRACT: &str = "inj1zwv6feuzhy6a9wekh96cd57lsarmqlwxvdl4nk";
    pub const CW20_CONTRACT: &str = "inj1h0y3hssxf4vsdacfmjg720642cvpxwyqh35kpn";
    pub const ADMIN: &str = "inj1qg5ega6dykkxc307y25pecuufrjkxkag6xhp6y";
    pub const USER: &str = "inj1gfawuv6fslzjlfa4v7exv27mk6rpfeyv823eu2";

    #[test]
    fn it_can_perform_basic_operations() {
        let mut deps = mock_dependencies();
        let mut wasm_querier = WasmMockQuerier::new();

        wasm_querier.balance_query_handler = create_simple_balance_bank_query_handler(vec![Coin::new(10, "inj")]);
        wasm_querier.smart_query_handler = create_cw20_info_query_handler();
        deps.querier = wasm_querier;

        let msg = InstantiateMsg {};

        let info_inst = mock_info(ADMIN, &[]);
        let _res_inst = instantiate(deps.as_mut(), mock_env(ADAPTER_CONTRACT), info_inst, msg).unwrap();

        // send some tokens to a contract
        let info_receive = mock_info(CW20_CONTRACT, &[]);
        let msg = ExecuteMsg::Receive {
            sender: USER.to_string(),
            amount: Uint128::new(1000),
            msg: Default::default(),
        };
        let _res_receive = execute(deps.as_mut(), mock_env(ADAPTER_CONTRACT), info_receive, msg).unwrap();

        let denom = get_denom(&Addr::unchecked(ADAPTER_CONTRACT), &Addr::unchecked(CW20_CONTRACT));
        // redeem some tokens to a contract
        let info_redeem = mock_info(USER, &[Coin::new(800, denom)]);
        let msg = ExecuteMsg::RedeemAndTransfer { recipient: None };
        let res_redeem = execute(deps.as_mut(), mock_env(ADAPTER_CONTRACT), info_redeem, msg);

        assert!(res_redeem.is_ok());
    }
}
