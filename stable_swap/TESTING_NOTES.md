# Testing Notes:


## Build 
Built using `forc 0.33.1`



Launch `fuel-core` with volatile db:
```
fuel-core run --db-type in-memory
```

Build and Deploy token contract:
```
nuclear_swap/token_contract$ forc build
nuclear_swap/token_contract$ forc deploy --unsigned
```

Build Swap contract:
```
nuclear_swap/stable_swap$ forc build
```


## Tests

Run all:
```
cargo test
```

with no output:
```
cargo test can_get_contract_id
cargo test can_deposit
cargo test can_get_balance
cargo test can_withdraw
cargo test can_add_liquidity
cargo test can_remove_liquidity
cargo test can_swap
cargo test can_add_liquidity_to_existing_supply
cargo test use_liquidity
```

Show output and logs for specific test:
```
cargo test can_get_contract_id -- --show-output
cargo test can_deposit -- --show-output
cargo test can_get_balance -- --show-output
cargo test can_withdraw -- --show-output
cargo test can_add_liquidity -- --show-output
cargo test can_remove_liquidity -- --show-output
cargo test can_swap -- --show-output
cargo test can_add_liquidity_to_existing_supply -- --show-output
cargo test use_liquidity -- --show-output

```