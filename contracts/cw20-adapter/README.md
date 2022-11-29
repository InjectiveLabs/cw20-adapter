# CW-20 Adapter

Contract that allows exchanging CW-20 tokens for injective-chain issued native tokens (using Token Factory module) and vice-versa.

## Background

CW-20 is a specification for fungible tokens in Cosmwasm, loosely based on ERC-20 specification. It allows creating and handling of arbitrary fungible tokens within Cosmwasm, specifying methods for creating, minting and burning and transferring those tokens between accounts.

While CW-20 is relatively mature and complete standard, the tokens exists purely within cosmwasm context and are entirely managed by the issuing contract (including keeping track of account balances). That means that they cannot interact directly with any Injective/Cosmos chain functionality (for example it’s not possible to trade them on Injective exchange, or transfer without involving issuing contract)

Considering the above, it’s necessary to provide some solution that would work as a bridge between CW20 and Injective bank module. 

### Goals

Adapter contract allows exchanging CW-20 tokens for injective-chain issued native tokens (using Token Factory module) and vice-versa. Adapter contract should assure that only authorized source CW-20 contracts can mint tokens (to avoid faking tokens).

Main functions of the contract are:
- register new CW-20 token
- exchange amount of X cw-20 tokens for X TF tokens (original cw-20 tokens will be held by the contract)
- exchange X TF tokens back for cw-20 tokens  (cw-20 tokens are released and TF tokens are burned)

## Messages 

### RegisterCw20Contract { addr: Addr }
Registers a new CW-20 contract (addr) that will be handled by the adapter and creates a new
TokenFactory token in format `factory/{adapter_contract}/{cw20_contract}`

Message must provide enough funds to create new TokenFactory denom (10 inj by default, but caller should query the 
chain for the current value)  

### Receive { sender: String, amount: Uint128, msg: Binary },
Implementation of Receiver CW-20 interface. Should be called by CW-20 contract only!! (never directly). 
Sender will contain address that initiated Send method on CW-20 contract 
Amount is amount of CW-20 tokens transferred 
Msg is ignored

Upon receiving this message, adapter will: 
- check if calling address is registered - if not and contract address has enough funds, it will register it (see above). 
- will mint and transfer to a `sender` address (original caller of cw20 send method) `amount` of TF tokens 

### RedeemAndTransfer { recipient: Option<String> }
Will redeem attached TF tokens (will fail if no registered tokens are provided)
and will transfer CW-20 tokens to `recipient`. If recipient is not provided, they will be sent 
to the message sender. 

This method uses CW-20 `transfer` method. It will not notify recipient in any way, so it's not advisable 
to use it to send tokens to a contract address. 

### RedeemAndSend { recipient: String, submessage: Binary }
Will redeem attached TF tokens (will fail if no registered tokens are provided)
and will send CW-20 tokens to `recipient` contract. Caller may provide optional submessage 

This method uses CW-20 `send` method, so the recipient must be a contract which adheres to cw20 Recipient specification,
and should be able to react properly to funds sent this way.  

### UpdateMetadata { addr : Addr} 
Will query cw20 address (if registered) for metadata and will call setMetadata in the bank module (using TokenFactory 
access method)
Warning: this require chain v1.9. Can be called any time

# Queries 

### RegisteredContracts {}
Return a list of registered CW-20 contracts

### NewDenomFee {}
Returns a fee required to register a new token-factory denom



