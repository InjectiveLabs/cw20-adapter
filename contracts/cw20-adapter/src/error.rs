use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] cosmwasm_std::StdError),

    #[error("No registered tokens provided")]
    NoRegisteredTokensProvided,

    #[error("CW-20 contract with the same address was already registered")]
    ContractAlreadyRegistered,

    #[error("CW-20 contract is not registered in adapter")]
    ContractNotRegistered,

    #[error("Adapter is missing balance to create a new token-factory denom")]
    NotEnoughBalanceToPayDenomCreationFee,

    #[error("Some of the provided funds are not required")]
    SuperfluousFundsProvided,

    #[error("Address is not cw-20 contract")]
    NotCw20Address,
}
