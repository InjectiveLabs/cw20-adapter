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
    RegisterCw20Contract { addr: Addr },
    ///  Impl of Receiver CW-20 interface. Should be called by CW-20 contract only!! (never directly). Msg is ignored
    Receive { sender: String, amount: Uint128, msg: Binary },
    /// Called to redeem TF tokens. Will send CW-20 tokens to "recipient" address (or sender if not provided). Will use transfer method
    RedeemAndTransfer { recipient: Option<String> },
    /// Called to redeem TF tokens. Will call Send method of CW:20 to send CW-20 tokens to "recipient" address. Submessage will be passed to send method (can be empty)
    RedeemAndSend { recipient: String, submsg: Binary },
    /// Updates stored metadata
    UpdateMetadata { addr: Addr },
}

#[cw_serde]
pub enum QueryMsg {
    /// Return a list of registered CW-20 contracts
    RegisteredContracts {},
    /// Returns a fee required to register a new token-factory denom
    NewDenomFee {},
}
