use fuel_tx::ContractId;
use fuels_abigen_macro::abigen;
use fuels::prelude::*;
use fuels::test_helpers;

// Load abi from json
abigen!(MyContract, "out/debug/stable_swap-abi.json");

async fn get_contract_instance() -> (MyContract, ContractId) {
    // Launch a local network and deploy the contract
    let wallet = launch_provider_and_get_wallet().await;

    let id = Contract::deploy("./out/debug/stable_swap.bin", &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance = MyContract::new(id.to_string(), wallet);

    (instance, id)
}

#[tokio::test]
async fn can_deposit() {
    let (_instance, _id) = get_contract_instance().await;

    _instance
        .deposit()
        .call_params(CallParameters::new(Some(11), None))
        .call()
        .await
        .unwrap();

    // Now you have an instance of your contract you can use to test each function
}

#[tokio::test]
async fn can_get_balance() {
    let (_instance, _id) = get_contract_instance().await;

    _instance
        .deposit()
        .call_params(CallParameters::new(Some(11), None))
        .call()
        .await
        .unwrap();

    // Native asset id
    let native_asset_id = ContractId::new(*NATIVE_ASSET_ID);

    let response = _instance
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    assert_eq!(response.value, 11);

    // Now you have an instance of your contract you can use to test each function
}