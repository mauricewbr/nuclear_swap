use fuel_tx::ContractId;
use fuels_abigen_macro::abigen;
//use fuels::prelude::launch_provider_and_get_wallet;
use fuels::prelude::*;
//use fuels::test_helpers;

// Load abi from json
abigen!(MyContract, "out/debug/stable_swap-abi.json");

#[tokio::test]
async fn contract() {
    let wallet = launch_provider_and_get_wallet().await;

    // Deploy contract and get ID
    let exchange_contract_id = Contract::deploy(
        "out/debug/stable_swap.bin",
            &wallet,
            TxParameters::default()
        )
        .await
        .unwrap();
    
    let exchange_instance = MyContract::new(
        exchange_contract_id.to_string(),
        wallet.clone()
    );
    

    exchange_instance
        .deposit()
        .call_params(CallParameters::new(Some(11), None))
        .call()
        .await
        .unwrap();

    // Native asset id
    let native_asset_id = ContractId::new(*NATIVE_ASSET_ID);

    // Check contract balance
    let response = exchange_instance
        .get_balance(native_asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(response.value, 11);

}