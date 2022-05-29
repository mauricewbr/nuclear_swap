use fuel_tx::{AssetId, ContractId};
use fuels_abigen_macro::abigen;
use fuels::prelude::*;
use fuels::test_helpers;

// Load abi from json
abigen!(MyContract, "out/debug/stable_swap-abi.json");
abigen!(TestToken,"../token_contract/out/debug/token_contract-abi.json");

async fn get_contract_instance() -> (MyContract, ContractId, TestToken, ContractId) {
    // Launch a local network and deploy the contract
    let wallet = launch_provider_and_get_wallet().await;

    let swap_contract_id = Contract::deploy("./out/debug/stable_swap.bin", &wallet, TxParameters::default())
        .await
        .unwrap();

    let swap_contract_instance = MyContract::new(swap_contract_id.to_string(), wallet.clone());

    // Get the contract ID and a handle to it
    let token_contract_id =
        Contract::deploy(
            "../token_contract/out/debug/token_contract.bin",
            &wallet,
            TxParameters::default()
        )
        .await
        .unwrap();
    let token_contract_instance = TestToken::new(token_contract_id.to_string(), wallet.clone());

    (swap_contract_instance, swap_contract_id, token_contract_instance, token_contract_id)
}

#[tokio::test]
async fn can_deposit() {
    let (_swap_contract_instance, _swap_contract_id, _token_contract_instance, _token_contract_id) = get_contract_instance().await;

    _swap_contract_instance
        .deposit()
        .call_params(CallParameters::new(Some(11), None))
        .call()
        .await
        .unwrap();

    // Now you have an instance of your contract you can use to test each function
}

#[tokio::test]
async fn can_get_balance() {
    let (_swap_contract_instance, _swap_contract_id, _token_contract_instance, _token_contract_id) = get_contract_instance().await;

    _swap_contract_instance
        .deposit()
        .call_params(CallParameters::new(Some(11), None))
        .call()
        .await
        .unwrap();

    // Native asset id
    let native_asset_id = ContractId::new(*NATIVE_ASSET_ID);

    let response = _swap_contract_instance
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    assert_eq!(response.value, 11);

    // Now you have an instance of your contract you can use to test each function
}

#[tokio::test]
async fn can_withdraw() {
    let (_swap_contract_instance, _swap_contract_id, _token_contract_instance, _token_contract_id) = get_contract_instance().await;

    // Depost some native assets
    _swap_contract_instance
        .deposit()
        .call_params(CallParameters::new(Some(11), None))
        .call()
        .await
        .unwrap();

    // Native asset id
    let native_asset_id = ContractId::new(*NATIVE_ASSET_ID);

    // Check contract balance
    let response = _swap_contract_instance
        .get_balance(native_asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(response.value, 11);

    _swap_contract_instance
        .withdraw(11, native_asset_id.clone())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();
    
    // Check contract balance
    let response = _swap_contract_instance
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    assert_eq!(response.value, 0);
}

#[tokio::test]
async fn can_add_liquidity() {
    // Launch a local network and deploy the contract
    let wallet = launch_provider_and_get_wallet().await;

    let _swap_contract_id = Contract::deploy("./out/debug/stable_swap.bin", &wallet, TxParameters::default())
        .await
        .unwrap();

    let _swap_contract_instance = MyContract::new(_swap_contract_id.to_string(), wallet.clone());

    // Get the contract ID and a handle to it
    let _token_contract_id =
        Contract::deploy(
            "../token_contract/out/debug/token_contract.bin",
            &wallet,
            TxParameters::default()
        )
        .await
        .unwrap();
    let _token_contract_instance = TestToken::new(_token_contract_id.to_string(), wallet.clone());

    // Mint some alt tokens
    _token_contract_instance.mint_coins(10000).call().await.unwrap();

    // Check the balance of the contract of its own asset
    let result = _token_contract_instance
        .get_balance(_token_contract_id.clone(), _token_contract_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, 10000);

    // Transfer some alt tokens to the wallet
    let address = wallet.address();
    let _t = _token_contract_instance
        .transfer_coins_to_output(50, _token_contract_id.clone(), address.clone())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check the balance of the contract of its own asset
    let result = _token_contract_instance
        .get_balance(_token_contract_id.clone(), _token_contract_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, 10000 - 50);

    let alt_token_id = AssetId::from(*_token_contract_id.clone());
    let lp_token_id = AssetId::from(*_swap_contract_id.clone());
    
    // Inspect the wallet for alt tokens
    let coins = wallet
        .get_spendable_coins(&alt_token_id, 50)
        .await
        .unwrap();
    assert_eq!(coins[0].amount, 50u64.into());
    
    // Deposit 50 native assets
    _swap_contract_instance
        .deposit()
        .call_params(CallParameters::new(Some(50), None))
        .call()
        .await
        .unwrap();

    // deposit 50 alt tokens into the Exchange contract
    _swap_contract_instance
        .deposit()
        .call_params(CallParameters::new(
            Some(50),
            Some(alt_token_id.clone()),
        ))
        .call()
        .await
        .unwrap();
    
    // Add initial liquidity, setting proportion 1:1
    // where lp tokens returned should be equal to the eth_amount deposited 50
    _swap_contract_instance
        .add_liquidity(1, 1000)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check LP tokens amount to be 50
    assert_eq!(
        wallet
            .get_spendable_coins(&lp_token_id, 50)
            .await
            .unwrap()[0]
            .amount,
        50u64.into()
    );

    // Fund the wallet again with some alt tokens
    _token_contract_instance
        .transfer_coins_to_output(100, _token_contract_id.clone(), address.clone())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Deposit 100 native assets
    _swap_contract_instance
        .deposit()
        .call_params(CallParameters::new(Some(100), None))
        .call()
        .await
        .unwrap();

    // Deposit 100 alt tokens
    _swap_contract_instance
        .deposit()
        .call_params(CallParameters::new(
            Some(100),
            Some(alt_token_id.clone()),
        ))
        .call()
        .await
        .unwrap();

    // Add liquidity for the second time. Keeping the proportion 1:1
    // It should return the same amount of LP as the amount of ETH deposited
    let result = _swap_contract_instance
        .add_liquidity(1, 1000)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, 100);

    // Inspect the wallet for LP tokens - should see 50 LP tokens + 100 LP tokens
    let lp_tokens = wallet
        .get_spendable_coins(&lp_token_id, 150)
        .await
        .unwrap();
    assert!(
        (lp_tokens[0].amount == 50u64.into()) && (lp_tokens[1].amount == 100u64.into())
        || (lp_tokens[0].amount == 100u64.into()) && (lp_tokens[1].amount == 50u64.into())
    );
}

#[tokio::test]
async fn can_remove_liquidity() {
    // Launch a local network and deploy the contract
    let wallet = launch_provider_and_get_wallet().await;

    let _swap_contract_id = Contract::deploy("./out/debug/stable_swap.bin", &wallet, TxParameters::default())
        .await
        .unwrap();

    let _swap_contract_instance = MyContract::new(_swap_contract_id.to_string(), wallet.clone());

    // Get the contract ID and a handle to it
    let _token_contract_id =
        Contract::deploy(
            "../token_contract/out/debug/token_contract.bin",
            &wallet,
            TxParameters::default()
        )
        .await
        .unwrap();
    let _token_contract_instance = TestToken::new(_token_contract_id.to_string(), wallet.clone());

    // Mint some alt tokens
    _token_contract_instance.mint_coins(10000).call().await.unwrap();

    // Check the balance of the contract of its own asset
    let result = _token_contract_instance
        .get_balance(_token_contract_id.clone(), _token_contract_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, 10000);

    // Transfer some alt tokens to the wallet
    let address = wallet.address();
    let _t = _token_contract_instance
        .transfer_coins_to_output(50, _token_contract_id.clone(), address.clone())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check the balance of the contract of its own asset
    let result = _token_contract_instance
        .get_balance(_token_contract_id.clone(), _token_contract_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, 10000 - 50);

    let alt_token_id = AssetId::from(*_token_contract_id.clone());
    let lp_token_id = AssetId::from(*_swap_contract_id.clone());
    
    // Inspect the wallet for alt tokens
    let coins = wallet
        .get_spendable_coins(&alt_token_id, 50)
        .await
        .unwrap();
    assert_eq!(coins[0].amount, 50u64.into());
    
    // Deposit 50 native assets
    _swap_contract_instance
        .deposit()
        .call_params(CallParameters::new(Some(50), None))
        .call()
        .await
        .unwrap();

    // deposit 50 alt tokens into the Exchange contract
    _swap_contract_instance
        .deposit()
        .call_params(CallParameters::new(
            Some(50),
            Some(alt_token_id.clone()),
        ))
        .call()
        .await
        .unwrap();
    
    // Add initial liquidity, setting proportion 1:1
    // where lp tokens returned should be equal to the eth_amount deposited 50
    _swap_contract_instance
        .add_liquidity(1, 1000)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check LP tokens amount to be 50
    assert_eq!(
        wallet
            .get_spendable_coins(&lp_token_id, 50)
            .await
            .unwrap()[0]
            .amount,
        50u64.into()
    );

    // Remove 30 native and 30 alt tokens 
    let result = _swap_contract_instance
        .remove_liquidity(30, 30, 1000)
        .call_params(CallParameters::new(
            Some(30),
            Some(lp_token_id.clone()),
        ))
        .append_variable_outputs(2)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value.eth_amount, 30);
    assert_eq!(result.value.token_amount, 30);
    
    // Inspect the wallet for LP tokens
    // It should have 20 lp tokens)
    let spendable_coins = wallet
        .get_spendable_coins(&lp_token_id, 20)
        .await
        .unwrap();
    let total_amount: u64 = spendable_coins.iter().map(|c| c.amount.0).sum();

    // Inspect the wallet for alt tokens to be 20
    assert_eq!(total_amount, 20);
}