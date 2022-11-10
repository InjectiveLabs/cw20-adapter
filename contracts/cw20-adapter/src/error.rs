use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] cosmwasm_std::StdError),

    #[error("No registered tokens provided")]
    NoRegisteredTokensProvided,

    #[error("CW-20 contract with the same address was already registered")]
    ContractAlreadyRegistered,


}
