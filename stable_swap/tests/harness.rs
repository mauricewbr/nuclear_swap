use fuel_tx::ContractId;
use fuels_abigen_macro::abigen;
use fuels::prelude::*;
use fuels::test_helpers;

// Load abi from json
abigen!(MyContract, "out/debug/stable_swap-abi.json");
abigen!(TestToken,"../token_contract/out/debug/token_contract-abi.json");

async fn get_contract_instance() -> (MyContract, ContractId, TestToken, ContractId) {
    // Launch a local network and deploy the contract
    let wallet = launch_provider_and_get_wallet().await;

    let contract_id = Contract::deploy("./out/debug/stable_swap.bin", &wallet, TxParameters::default())
        .await
        .unwrap();

    let contract_instance = MyContract::new(contract_id.to_string(), wallet.clone());

    // Get the contract ID and a handle to it
    let token_id =
        Contract::deploy(
            "../token_contract/out/debug/token_contract.bin",
            &wallet,
            TxParameters::default()
        )
        .await
        .unwrap();
    let token_instance = TestToken::new(token_id.to_string(), wallet.clone());

    (contract_instance, contract_id, token_instance, token_id)
}

#[tokio::test]
async fn can_deposit() {
    let (_contract_instance, _contract_id, _token_instance, _token_id) = get_contract_instance().await;

    _contract_instance
        .deposit()
        .call_params(CallParameters::new(Some(11), None))
        .call()
        .await
        .unwrap();

    // Now you have an instance of your contract you can use to test each function
}

#[tokio::test]
async fn can_get_balance() {
    let (_contract_instance, _contract_id, _token_instance, _token_id) = get_contract_instance().await;

    _contract_instance
        .deposit()
        .call_params(CallParameters::new(Some(11), None))
        .call()
        .await
        .unwrap();

    // Native asset id
    let native_asset_id = ContractId::new(*NATIVE_ASSET_ID);

    let response = _contract_instance
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    assert_eq!(response.value, 11);

    // Now you have an instance of your contract you can use to test each function
}

#[tokio::test]
async fn can_withdraw() {
    let (_contract_instance, _contract_id, _token_instance, _token_id) = get_contract_instance().await;

    // Depost some native assets
    _contract_instance
        .deposit()
        .call_params(CallParameters::new(Some(11), None))
        .call()
        .await
        .unwrap();

    // Native asset id
    let native_asset_id = ContractId::new(*NATIVE_ASSET_ID);

    // Check contract balance
    let response = _contract_instance
        .get_balance(native_asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(response.value, 11);

    _contract_instance
        .withdraw(11, native_asset_id.clone())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();
    
    // Check contract balance
    let response = _contract_instance
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    assert_eq!(response.value, 0);
}

#[tokio::test]
async fn can_add_liquidity() {
    let (_contract_instance, _contract_id, _token_instance, _token_id) = get_contract_instance().await;

}