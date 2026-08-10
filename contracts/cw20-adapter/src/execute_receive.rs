use crate::common::{get_denom, is_contract_registered};
use crate::error::ContractError;
use cosmwasm_std::{Coin, DepsMut, Env, MessageInfo, Response, Uint128};
use injective_cosmwasm::{create_mint_tokens_msg, InjectiveMsgWrapper, InjectiveQueryWrapper};

pub fn handle_on_received_cw20_funds_msg(
    deps: DepsMut<InjectiveQueryWrapper>,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response<InjectiveMsgWrapper>, ContractError> {
    if !info.funds.is_empty() {
        return Err(ContractError::SuperfluousFundsProvided);
    }

    let token_contract = info.sender;

    if !is_contract_registered(&deps, &token_contract) {
        return Err(ContractError::ContractNotRegistered);
    }

    let master = env.contract.address;

    let denom = get_denom(&master, &token_contract);
    let coins_to_mint = Coin::new(amount.u128(), denom);
    let mint_tf_tokens_message = create_mint_tokens_msg(master, coins_to_mint, recipient);

    Ok(Response::new().add_message(mint_tf_tokens_message))
}
