# ABI Generation Note

## Issue
The fusion-plus-htlc contract currently cannot generate ABI using `cargo near build` due to JsonSchema trait implementation issues with NEAR SDK 5.x types,
 specifically:
- `AccountId` does not implement `JsonSchema`
- Complex types containing `AccountId` fail ABI generation

## Temporary Workaround
The contract is built using a custom build script (`build-without-abi.sh`) that:
1. Builds the contract using standard `cargo build --target wasm32-unknown-unknown --release`
2. Copies the resulting WASM to the expected location
3. Skips ABI generation

## Impact
- The contract functions normally and can be deployed
- ABI must be manually specified when interacting with the contract
- This is a known limitation of NEAR SDK 5.x when using `AccountId` in complex return types

## Future Solution
When NEAR SDK provides JsonSchema support for AccountId, or when we migrate to wrapper types, we can:
1. Remove the custom build script
2. Re-enable standard `cargo near build`
3. Generate ABI automatically

## Contract Methods (Manual ABI Reference)
- `new(owner: AccountId)`: Initialize contract
- `create_htlc(args: HTLCCreateArgs)`: Create new HTLC
- `withdraw(htlc_id: String, secret: String)`: Withdraw with secret
- `refund(htlc_id: String)`: Refund after timeout
- `get_htlc(htlc_id: String)`: Query HTLC details
- `get_stats()`: Get contract statistics
- `add_supported_token(token: AccountId)`: Add token support
- `ft_on_transfer(sender_id: AccountId, amount: U128, msg: String)`: NEP-141 receiver