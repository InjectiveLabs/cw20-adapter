use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct ReceiveSubmsg {
    pub(crate) recipient: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Registers a new CW-20 contract that will be handled by the adapter
    RegisterCw20Contract {
        addr: Addr,
    },
    ///  Impl of Receiver CW-20 interface
    Receive {
        sender: String,
        amount: Uint128,
        msg: Binary,
    },
    Redeem {
        recipient: String,
    },
}

#[cw_serde]
pub enum QueryMsg {
    /// Return a list of registered CW-20 contracts
    RegisteredContracts {},
    /// Returns a fee required to register a new token-factory denom
    NewDenomFee {},
}
