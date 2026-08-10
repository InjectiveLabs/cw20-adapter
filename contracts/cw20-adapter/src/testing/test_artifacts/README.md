# Test artifacts

`cw20_adapter_v1.wasm` is the exact legacy adapter bytecode stored as code ID 8
on `injective-1` and used by contract
`inj14ejqjyq8um4p3xfqj74yld5waqljf88f9eneuk`.

- Source endpoint: `https://sentry.lcd.injective.network/cosmwasm/wasm/v1/code/8`
- SHA-256: `16d6acc2dce21f7e998dccf4e1d0a08966a056c5a0227a220cb69d257d620301`

The migration test stores and instantiates this fixture in `InjectiveTestApp`,
creates legacy registry and reserve state, migrates it to the current workspace
Wasm, then verifies the version, state, and redemption path.
