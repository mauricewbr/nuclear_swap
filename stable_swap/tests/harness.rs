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
}