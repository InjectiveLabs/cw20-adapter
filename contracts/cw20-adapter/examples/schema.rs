use cosmwasm_schema::write_api;

use cw20_adapter::msg;

fn main() {
    write_api! {
        instantiate: hub::InstantiateMsg,
        sudo: hub::SudoMsg,
        execute: hub::ExecuteMsg,
        query: hub::QueryMsg,
    }
}
