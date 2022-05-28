//use fuel_tx::ContractId;
//use fuels_abigen_macro::abigen;
use fuels::prelude::launch_provider_and_get_wallet;
//use fuels::test_helpers;

// Load abi from json
//abigen!(MyContract, "out/debug/ns_lib-abi.json");

#[tokio::test]
async fn can_get_contract_id() {
    let _wallet = launch_provider_and_get_wallet().await;
    ()
    
}