use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] cosmwasm_std::StdError),

    #[error("No registered tokens provided")]
    NoRegisteredTokensProvided,

    #[error("CW-20 contract with the same address was already registered")]
    ContractAlreadyRegistered,

    #[error("Adapter is missing balance to create a new token-factory denom")]
    NotEnoughBalanceToPayDenomCreationFee,
}
