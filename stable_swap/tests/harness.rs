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

    exchange_instance
        .withdraw(11, native_asset_id.clone())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check contract balance
    let response = exchange_instance
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    assert_eq!(response.value, 0);


    exchange_instance.mint_coins(10000).call().await.unwrap();

    let result = exchange_instance
        .get_balance_asset(exchange_contract_id.clone(), exchange_contract_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, 10000);

    //////////////////////////////////////////
    // Start transferring and adding liquidity
    //////////////////////////////////////////

    // Transfer some alt tokens to the wallet
    let address = wallet.address();
    let _t = exchange_instance
        .transfer_coins_to_output(50, exchange_contract_id.clone(), address.clone())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

}