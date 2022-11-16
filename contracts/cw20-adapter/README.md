# CW-20 Adapter

Contract that allows exchanging CW-20 tokens for injective-chain issued native tokens (using Token Factory module) and vice-versa.

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
to the message caller. 

This method uses CW-20 `transfer` method (so it will not notify recipient in any way)

### RedeemAndSend { recipient: String, submessage: Binary }
Will redeem attached TF tokens (will fail if no registered tokens are provided)
and will send CW-20 tokens to `recipient` contract. Caller may provide optional submessage 

This method uses CW-20 `send` method


# Queries 

### RegisteredContracts {}
Return a list of registered CW-20 contracts

### NewDenomFee {}
Returns a fee required to register a new token-factory denom
