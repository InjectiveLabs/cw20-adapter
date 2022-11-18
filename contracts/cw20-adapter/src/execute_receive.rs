use crate::common::{ensure_sufficient_create_denom_balance, get_denom, is_contract_registered, register_contract_and_get_message};
use crate::error::ContractError;
use cosmwasm_std::{Coin, DepsMut, Env, MessageInfo, Response, Uint128};
use injective_cosmwasm::{create_mint_tokens_msg, InjectiveMsgWrapper, InjectiveQueryWrapper};

pub fn handle_on_received_cw20_funds_msg(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    if !info.funds.is_empty() {
        return Err(ContractError::SuperfluousFundsProvided);
    }
    let mut response = Response::new();
    let contract = info.sender;
    if !is_contract_registered(&deps, &contract) {
        ensure_sufficient_create_denom_balance(&deps, &env)?;
        response = response.add_message(register_contract_and_get_message(deps, &env, &contract)?);
    }
    let master = env.contract.address;

    let denom = get_denom(&master, &contract);
    let coins_to_mint = Coin::new(amount.u128(), denom);
    let mint_tf_tokens_message = create_mint_tokens_msg(master, coins_to_mint, sender);

    Ok(response.add_message(mint_tf_tokens_message))
}
