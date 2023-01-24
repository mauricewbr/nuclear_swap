use fuels::{
    prelude::*,
    tx::{
        Address,
        ContractId,
        AssetId,
    }, 
    types::Bits256,
};
use rand::prelude::{Rng};

use more_asserts as ma;


abigen!(
    Contract(name="StableSwapContract", abi="./out/debug/stable_swap-abi.json"),
    Contract(name="TestToken", abi="../token_contract/out/debug/token_contract-abi.json"),
);


async fn get_contract_instance() -> (StableSwapContract, ContractId, TestToken, ContractId) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;
    let wallet = wallets.pop().unwrap();


    //-------------------------------------------------
    // get swap contract instance and id:
    //
    let swap_contract_id = Contract::deploy(
        "./out/debug/stable_swap.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "./out/debug/stable_swap-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let swap_contract_instance = StableSwapContract::new(swap_contract_id.clone(), wallet.clone());


    //-------------------------------------------------
    // get swap contract instance and id:
    //
    let token_contract_id = Contract::deploy(
        "../token_contract/out/debug/token_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "../token_contract/out/debug/token_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let token_contract_instance = TestToken::new(token_contract_id.clone(), wallet);


    //-------------------------------------------------
    // return swap contract instance/id & token contract instance/id:
    //
    (swap_contract_instance, swap_contract_id.into(), token_contract_instance, token_contract_id.into())
}





//-------------------------------------------------------------------------------------
// Tests:

#[tokio::test]
async fn can_get_contract_id() {
    let (
        _swap_contract_instance, 
        _swap_contract_id, 
        _token_contract_instance, 
        _token_contract_id
    ) = get_contract_instance().await;

    // Now you have an instance of your contract you can use to test each function
}



#[tokio::test]
async fn can_deposit() {
    let (_swap_contract_instance, _swap_contract_id, _token_contract_instance, _token_contract_id) = get_contract_instance().await;

    let deposit_amount = 1_000_001;
    let call_params = CallParameters::new(Some(deposit_amount), Some(BASE_ASSET_ID), None);

    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(call_params)
        .call()
        .await
        .unwrap();
}



#[tokio::test]
async fn can_get_balance() {
    let (_swap_contract_instance, _swap_contract_id, _token_contract_instance, _token_contract_id) = get_contract_instance().await;

    let deposit_amount = 1_000_002;
    let call_params = CallParameters::new(Some(deposit_amount), Some(BASE_ASSET_ID), None);
  

    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(call_params)
        .call()
        .await
        .unwrap();


    // Native asset id
    let native_asset_id = ContractId::new(*BASE_ASSET_ID);

    let response = _swap_contract_instance
        .methods()
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();

    println!("");
    println!("balance = {}", response.value);

    assert_eq!(response.value, 1_000_002);

}




#[tokio::test]
async fn can_withdraw() {
    println!("TEST: can_withdraw()");

    let (_swap_contract_instance, _swap_contract_id, _token_contract_instance, _token_contract_id) = get_contract_instance().await;

    let deposit_amount = 1_000_003;
    let call_params = CallParameters::new(Some(deposit_amount), Some(BASE_ASSET_ID), None);
  
    // Deposit some native assets
    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(call_params)
        .call()
        .await
        .unwrap();

    // Native asset id
    let native_asset_id = ContractId::new(*BASE_ASSET_ID);

    // Check contract balance
    let response = _swap_contract_instance
        .methods()
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();

    println!("");
    println!("balance = {}", response.value);

    assert_eq!(response.value, 1_000_003);

    _swap_contract_instance
        .methods()
        .withdraw(1_000_003, native_asset_id.clone())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();
    
    // Check contract balance
    let response = _swap_contract_instance
        .methods()
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();

    println!("after withdraw:");
    println!("balance = {}", response.value);

    assert_eq!(response.value, 0);

}





#[tokio::test]
async fn can_add_liquidity() {
    println!("TEST: can_add_liquidity()");

    // Launch a local network and deploy the contract
    let wallet = launch_provider_and_get_wallet().await;

    println!("wallet.address bech32: {}", wallet.address().to_string());
    println!("wallet address 0x    : {}", Address::from(wallet.address()));

    //--------------------------- 
    // get swap contract id/instance

    let _swap_contract_id_bech32 = Contract::deploy(
        "./out/debug/stable_swap.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "./out/debug/stable_swap-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let _swap_contract_instance = StableSwapContract::new(_swap_contract_id_bech32.clone(), wallet.clone());
    let _swap_contract_id = ContractId::from(_swap_contract_id_bech32);


    //--------------------------- 
    // Get the token contract ID and a handle to it

    let _token_contract_id_bech32 = Contract::deploy(
        "../token_contract/out/debug/token_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "../token_contract/out/debug/token_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let _token_contract_instance = TestToken::new(_token_contract_id_bech32.clone(), wallet.clone());
    let _token_contract_id = ContractId::from(_token_contract_id_bech32);
    println!("alt token contract id = {}", Address::from(*_token_contract_id.clone())); 


    // Mint some alt tokens
    _token_contract_instance
        .methods()
        .mint_coins(10000)
        .call()
        .await.
        unwrap();

    // Check the balance of the contract of its own asset
    let result = _token_contract_instance
        .methods()
        .get_balance(_token_contract_id.clone(), _token_contract_id.clone())
        .call()
        .await
        .unwrap();

    println!("get balance from mint:");
    println!("token contract balance = {}", result.value);        
    assert_eq!(result.value, 10000);


    // Transfer some alt tokens to the wallet
    let address = Address::from(wallet.address());

    let _t = _token_contract_instance
        .methods()
        .transfer_coins_to_output(50, _token_contract_id.clone(), address.clone())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check the balance of the contract of its own asset
    let result = _token_contract_instance
        .methods()
        .get_balance(_token_contract_id.clone(), _token_contract_id.clone())
        .call()
        .await
        .unwrap();


    println!("get alt token balance of contract:");
    println!("alt token balance = {}", result.value);      
    assert_eq!(result.value, 10000 - 50);

    let alt_token_id = AssetId::from(*_token_contract_id.clone());
    let lp_token_id = AssetId::from(*_swap_contract_id.clone());

    // Inspect the wallet for alt tokens
    let coins = wallet
        .get_asset_balance(&alt_token_id)
        .await
        .unwrap();

    println!("wallet alt token balance = {}", coins); 
    assert_eq!(coins,  <u64 as Into<u64>>::into(50u64));

    println!("add liquidity 50/50 get back LP tokens:");
    // Deposit 50 native assets
    let deposit_amount_native_a = 50;
    let call_params_a = CallParameters::new(Some(deposit_amount_native_a), Some(BASE_ASSET_ID), None);
    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(call_params_a)
        .call()
        .await
        .unwrap();

    // deposit 50 alt tokens into the Exchange contract
    let deposit_amount_alt_b = 50;
    let call_params_b = CallParameters::new(Some(deposit_amount_alt_b), Some(alt_token_id.clone()), None);
    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(call_params_b)
        .call()
        .await
        .unwrap();
    
    // Add initial liquidity, setting proportion 1:1
    // where lp tokens returned should be equal to the eth_amount deposited 50
    _swap_contract_instance
        .methods()
        .add_liquidity(1, 1000)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check LP tokens amount to be 50
    let wallet_lp_balance: u64 = wallet.get_asset_balance(&lp_token_id).await.unwrap();
    println!("wallet LP token balance = {}", wallet_lp_balance.clone()); 
    assert_eq!( wallet_lp_balance, <u64 as Into<u64>>::into(50u64) );

    println!("----------------------------------------");
    print!("\nAdd liquidity for the second time. Keeping the proportion 1:1 \n\
     It should return the same amount of LP as the amount of ETH deposited:\n");

    // Fund the wallet again with some alt tokens
    let _t2 = _token_contract_instance
        .methods()
        .transfer_coins_to_output(100, _token_contract_id.clone(), Address::from(wallet.address()))
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Deposit 100 native assets
    let deposit_amount_native_c = 100;
    let call_params_c = CallParameters::new(Some(deposit_amount_native_c), Some(BASE_ASSET_ID), None);
    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(call_params_c)
        .call()
        .await
        .unwrap();
 
    // deposit 100 alt tokens into the Exchange contract
    let deposit_amount_alt_d = 100;
    let call_params_d = CallParameters::new(Some(deposit_amount_alt_d), Some(alt_token_id.clone()), None);
    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(call_params_d)
        .call()
        .await
        .unwrap();

    // Add liquidity again and check that the LP tokens received equals amount of ETH deposited
    let _result = _swap_contract_instance
        .methods()
        .add_liquidity(1, 1000)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();
 
    // Inspect the wallet for LP tokens - should see 50 LP tokens + 100 LP tokens
    let lp_tokens_total = wallet
        .get_asset_balance(&lp_token_id)
        .await
        .unwrap();

  
    println!("wallet Native balance   = {}", wallet.get_asset_balance(&BASE_ASSET_ID).await.unwrap()); 
    println!("wallet alt token balance = {}", wallet.get_asset_balance(&alt_token_id).await.unwrap());
    println!("wallet LP token balance = {}", wallet.get_asset_balance(&lp_token_id).await.unwrap()); 
    println!("----------------------------------------");

    assert!( lp_tokens_total == <u64 as Into<u64>>::into(150u64) );

}




#[tokio::test]
async fn can_remove_liquidity() {
    println!("TEST: can_remove_liquidity()");

    // Launch a local network and deploy the contract
    let wallet = launch_provider_and_get_wallet().await;

    println!("wallet.address bech32: {}", wallet.address().to_string());
    println!("wallet address 0x    : {}", Address::from(wallet.address()));

    //--------------------------- 
    // get swap contract id/instance

    let _swap_contract_id_bech32 = Contract::deploy(
        "./out/debug/stable_swap.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "./out/debug/stable_swap-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let _swap_contract_instance = StableSwapContract::new(_swap_contract_id_bech32.clone(), wallet.clone());
    let _swap_contract_id = ContractId::from(_swap_contract_id_bech32);


    //--------------------------- 
    // Get the token contract ID and a handle to it

    let _token_contract_id_bech32 = Contract::deploy(
        "../token_contract/out/debug/token_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "../token_contract/out/debug/token_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let _token_contract_instance = TestToken::new(_token_contract_id_bech32.clone(), wallet.clone());
    let _token_contract_id = ContractId::from(_token_contract_id_bech32);
    println!("alt token contract id = {}", Address::from(*_token_contract_id.clone())); 

    // Mint some alt tokens
    _token_contract_instance
        .methods()
        .mint_coins(3000)
        .call()
        .await.
        unwrap();

    println!("Mint some alt tokens:");

    // Check the balance of the contract of its own asset
    let result = _token_contract_instance
        .methods()
        .get_balance(_token_contract_id.clone(), _token_contract_id.clone())
        .call()
        .await
        .unwrap();

    println!("token contract balance = {}", result.value);        
    assert_eq!(result.value, 3000);

    // Transfer some alt tokens to the wallet
    let address = Address::from(wallet.address());

    let _t = _token_contract_instance
        .methods()
        .transfer_coins_to_output(50, _token_contract_id.clone(), address.clone())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check the balance of the contract of its own asset
    let result = _token_contract_instance
        .methods()
        .get_balance(_token_contract_id.clone(), _token_contract_id.clone())
        .call()
        .await
        .unwrap();

    println!("get alt token balance of contract = {}", result.value);      
    assert_eq!(result.value, 3000 - 50);

    let alt_token_id = AssetId::from(*_token_contract_id.clone());
    let lp_token_id = AssetId::from(*_swap_contract_id.clone());

    // Inspect the wallet for alt tokens
    let coins = wallet
        .get_asset_balance(&alt_token_id)
        .await
        .unwrap();

    println!("wallet alt token balance = {}", coins); 
    assert_eq!(coins,  <u64 as Into<u64>>::into(50u64));

    println!("add liquidity 50/50 get back LP tokens:");
    // Deposit 50 native assets
    let deposit_amount_native_a = 50;
    let call_params_a = CallParameters::new(Some(deposit_amount_native_a), Some(BASE_ASSET_ID), None);
    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(call_params_a)
        .call()
        .await
        .unwrap();

    // deposit 50 alt tokens into the Exchange contract
    let deposit_amount_alt_b = 50;
    let call_params_b = CallParameters::new(Some(deposit_amount_alt_b), Some(alt_token_id.clone()), None);
    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(call_params_b)
        .call()
        .await
        .unwrap();
    
    // Add initial liquidity, setting proportion 1:1
    // where lp tokens returned should be equal to the eth_amount deposited 50
    _swap_contract_instance
        .methods()
        .add_liquidity(1, 1000)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check LP tokens amount to be 50
    let wallet_lp_balance: u64 = wallet.get_asset_balance(&lp_token_id).await.unwrap();
    println!("wallet LP token balance = {}", wallet_lp_balance.clone()); 
    assert_eq!( wallet_lp_balance, <u64 as Into<u64>>::into(50u64) );

    // Remove 30 native and 30 alt tokens 
    let remove_liq_amount = 30;
    let call_params_liq_remove = CallParameters::new(Some(remove_liq_amount), Some(lp_token_id.clone()), None);

    let result = _swap_contract_instance
        .methods()
        .remove_liquidity(30, 30, 1000)
        .call_params(call_params_liq_remove)
        .append_variable_outputs(2)
        .call()
        .await
        .unwrap();

    assert_eq!(result.value.eth_amount, 30);
    assert_eq!(result.value.token_amount, 30);

    println!("----------------------------------------");  
    println!("result.value.eth_amount   = {}", result.value.eth_amount); 
    println!("result.value.token_amount = {}", result.value.token_amount); 
 
    // Inspect the wallet for LP tokens
    // It should have 20 lp tokens
    let spendable_coins: u64 = wallet.get_asset_balance(&lp_token_id).await.unwrap();

    // Inspect the wallet for LP tokens to be 20
    assert_eq!( spendable_coins, <u64 as Into<u64>>::into(20u64) );

    println!(" ");
    println!("wallet Native balance   = {}", wallet.get_asset_balance(&BASE_ASSET_ID).await.unwrap()); 
    println!("wallet alt token balance = {}", wallet.get_asset_balance(&alt_token_id).await.unwrap());
    println!("wallet LP token balance = {}", wallet.get_asset_balance(&lp_token_id).await.unwrap()); 
    println!("----------------------------------------");   


}



//---------------------------------------------------------------------------------------------------------------------

async fn get_contract_instance_and_wallets() -> (
    StableSwapContract, 
    StableSwapContract,
    ContractId, 
    TestToken, 
    ContractId, 
    WalletUnlocked, 
    WalletUnlocked
) {
    
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(2),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;
    let wallet = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    //-------------------------------------------------
    // get swap contract instance and id:
    //

    let mut rng = rand::thread_rng();
    let salt = rng.gen::<[u8; 32]>();

    let swap_contract_id = Contract::deploy_with_parameters(
        "./out/debug/stable_swap.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "./out/debug/stable_swap-storage_slots.json".to_string())),
        Salt::from(salt),
    )
    .await
    .unwrap();


    let swap_contract_instance = StableSwapContract::new(swap_contract_id.clone(), wallet.clone());
    let swap_contract_instance2 = StableSwapContract::new(swap_contract_id.clone(), wallet2.clone());
    //-------------------------------------------------
    // get swap contract instance and id:
    //
    let token_contract_id = Contract::deploy(
        "../token_contract/out/debug/token_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "../token_contract/out/debug/token_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let token_contract_instance = TestToken::new(token_contract_id.clone(), wallet.clone());

    //-------------------------------------------------
    // return swap contract instance/id & token contract instance/id:
    //
    (swap_contract_instance, 
        swap_contract_instance2, 
        swap_contract_id.into(), 
        token_contract_instance, 
        token_contract_id.into(), 
        wallet, 
        wallet2)
}


async fn setup_system_for_test(
    alt_mint_amnt: u64, 
    alt_to_wal_amnt: u64) -> (
        StableSwapContract, 
        StableSwapContract,
        ContractId, 
        TestToken, 
        ContractId, 
        WalletUnlocked, 
        WalletUnlocked,
    ) {

    // launch and get contract instances
    let (
        _swap_contract_instance, 
        _swap_contract_instance2,
        _swap_contract_id, 
        _token_contract_instance, 
        _token_contract_id,
        _wallet,
        _wallet2,
    ) = get_contract_instance_and_wallets().await;

    println!("alt token contract id = {}", Address::from(*_token_contract_id.clone())); 

    println!("wallet.address bech32: {}", _wallet.address().to_string());
    println!("wallet address 0x    : {}", Address::from(_wallet.address()));

    //-----------------------
    // Mint some alt tokens
    _token_contract_instance
        .methods()
        .mint_coins(alt_mint_amnt)
        .call()
        .await.
        unwrap();
    println!("Mint some alt tokens:");

    // Check the balance of the contract of its own asset
    let result = _token_contract_instance
        .methods()
        .get_balance(_token_contract_id.clone(), _token_contract_id.clone())
        .call()
        .await
        .unwrap();
    println!("token contract balance = {}", result.value);        
    assert_eq!(result.value, alt_mint_amnt);

    // Transfer some alt tokens to the wallet
    let _t = _token_contract_instance
        .methods()
        .transfer_coins_to_output(
            alt_to_wal_amnt, 
            _token_contract_id.clone(), 
            Address::from(_wallet.address())
        )
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check the balance of the contract of its own asset
    let result = _token_contract_instance
        .methods()
        .get_balance(_token_contract_id.clone(), _token_contract_id.clone())
        .call()
        .await
        .unwrap();
    println!("get alt token balance of contract = {}", result.value);      
    assert_eq!(result.value, alt_mint_amnt - alt_to_wal_amnt);

    // Inspect the wallet for alt tokens
    let alt_coins = _wallet
        .get_asset_balance(&AssetId::from(*_token_contract_id.clone()))
        .await
        .unwrap();
    println!("wallet alt token balance = {}", alt_coins); 
    assert_eq!(alt_coins,  alt_to_wal_amnt);

    println!("finished setting up contracts/wallets for test.");
    (_swap_contract_instance, 
        _swap_contract_instance2, 
        _swap_contract_id.into(), 
        _token_contract_instance, 
        _token_contract_id.into(), 
        _wallet, _wallet2)

}



#[tokio::test]
async fn can_swap() {
    println!("TEST: can_swap()");

    let (
        _swap_contract_instance, 
        _swap_contract_instance2,
        _swap_contract_id, 
        _token_contract_instance, 
        _token_contract_id,
        _wallet,
        _wallet2,
    ) = setup_system_for_test(1000000, 500000).await;

    let alt_token_id = AssetId::from(*_token_contract_id.clone());
    let lp_token_id = AssetId::from(*_swap_contract_id.clone());
    let native_token_id = AssetId::from(*BASE_ASSET_ID);
    // Native asset id
    let native_asset_id = ContractId::new(*BASE_ASSET_ID);
    // Alt asset id
    let alt_asset_id = ContractId::new(*alt_token_id);
    

    println!("alt_token_id = {}", Address::from(*alt_token_id.clone())); 
    println!("lp_token_id = {}", Address::from(*lp_token_id.clone())); 
    println!("native_token_id = {}", Address::from(*native_token_id.clone())); 
    println!("native_asset_id = {}", Address::from(*native_asset_id.clone())); 
    println!("alt_asset_id = {}", Address::from(*alt_asset_id.clone())); 


    println!("Deposit 50000 native assets into the Exchange contract.");
    // Deposit 50000 native assets into the Exchange contract.
    let log = _swap_contract_instance
        .methods()
        .deposit()
        .call_params(
            CallParameters::new(Some(50000), 
            Some(BASE_ASSET_ID), 
            None)
        )
        .call()
        .await
        .unwrap();

    let response = _swap_contract_instance
        .methods()
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract, Native token balance: {:?}", response.value);

    println!("deposit 50000 alt tokens into the Exchange contract.");
    // deposit 50000 alt tokens into the Exchange contract
    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(CallParameters::new(
            Some(50000), 
            Some(alt_token_id.clone()), 
            None)
        )
        .call()
        .await
        .unwrap();
    
    let response = _swap_contract_instance
        .methods()
        .get_balance(alt_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract, Alt token balance: {:?}", response.value);

    println!(" ");
    println!("Add liquidity 1:1 get back LP tokens:");
    // Add initial liquidity, setting proportion 1:1
    // where lp tokens returned should be equal to the eth_amount deposited (50000 in this case)
    _swap_contract_instance
        .methods()
        .add_liquidity(1, 1000)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();


    // Check balances within swap contract:
    // get balances of swap contract
    let response = _swap_contract_instance
        .methods()
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract Native token balance: {:?}", response.value);

    let response = _swap_contract_instance
        .methods()
        .get_balance(alt_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract Alt token balance: {:?}", response.value);
    
    // Logging the token reserves after the add_liquidity
    println!("Swap contract After add_liquidity: {:?}", log.get_logs());

    // Check LP tokens amount to be = 50000
    let wallet_lp_balance: u64 = _wallet.get_asset_balance(&lp_token_id).await.unwrap();
    println!("Wallet1 LP token balance = {}", wallet_lp_balance.clone()); 
    assert_eq!( wallet_lp_balance, <u64 as Into<u64>>::into(50000u64) );

    // Inspect the wallet for alt tokens
    let _wallet_alt_bal = _wallet.get_asset_balance(&alt_token_id).await.unwrap();
    println!("wallet1 Alt token balance : {:?}", _wallet_alt_bal.clone());
    assert_eq!(_wallet_alt_bal, <u64 as Into<u64>>::into(450000u64) );


    //---------------------------------------------------


    let result_native = _swap_contract_instance
        .methods()
        .swap(1000, 20)
        .call_params(CallParameters::new(
            Some(1001),
            Some(native_token_id.clone()),
            None))
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();
    assert!(result_native.value > 0);
    println!("Result is {}", result_native.value);
    println!("Token Native and Token Alt BEFORE and AFTER the swap: {:?}", result_native.get_logs() );


    // let log_u64 = response.get_logs_with_type::<u64>();
    // println!("\n\nlog u64 {:?}\n\n", log_u64);

    let balances = _swap_contract_instance
        .methods()
        .get_balances(_swap_contract_id, _swap_contract_id)
        .call()
        .await
        .unwrap();
    println!("All swap contract balances: {:?}\n", balances);



}





#[tokio::test]
async fn can_add_liquidity_to_existing_supply() {
    println!("TEST: can_add_liquidity_to_existing_supply()");

    let (
        _swap_contract_instance, 
        _swap_contract_instance2,
        _swap_contract_id, 
        _token_contract_instance, 
        _token_contract_id,
        _wallet,
        _wallet2,
    ) = setup_system_for_test(1000000, 500000).await;


    //--------------------------------------------------
    let alt_token_id = AssetId::from(*_token_contract_id.clone());
    let lp_token_id = AssetId::from(*_swap_contract_id.clone());
    let native_token_id = AssetId::from(*BASE_ASSET_ID);
    // Native asset id
    let native_asset_id = ContractId::new(*BASE_ASSET_ID);
    // Alt asset id
    let alt_asset_id = ContractId::new(*alt_token_id);
    
    println!("alt_token_id    = {}", Address::from(*alt_token_id.clone())); 
    println!("lp_token_id     = {}", Address::from(*lp_token_id.clone())); 
    println!("native_token_id = {}", Address::from(*native_token_id.clone())); 
    println!("native_asset_id = {}", Address::from(*native_asset_id.clone())); 
    println!("alt_asset_id    = {}", Address::from(*alt_asset_id.clone())); 

    println!("Deposit 50000 native assets into the Exchange contract.");
    // Deposit 50000 native assets into the Exchange contract.
    let log = _swap_contract_instance
        .methods()
        .deposit()
        .call_params(
            CallParameters::new(Some(50000), 
            Some(BASE_ASSET_ID), 
            None)
        )
        .call()
        .await
        .unwrap();

    let response = _swap_contract_instance
        .methods()
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract, Native token balance: {:?}", response.value);

    println!("deposit 50000 alt tokens into the Exchange contract.");
    // deposit 50000 alt tokens into the Exchange contract
    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(CallParameters::new(
            Some(50000), 
            Some(alt_token_id.clone()), 
            None)
        )
        .call()
        .await
        .unwrap();
    
    let response = _swap_contract_instance
        .methods()
        .get_balance(alt_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract, Alt token balance: {:?}", response.value);

    println!(" ");
    println!("Add liquidity 1:1 get back LP tokens:");
    // Add initial liquidity, setting proportion 1:1
    // where lp tokens returned should be equal to the eth_amount deposited (50000 in this case)
    _swap_contract_instance
        .methods()
        .add_liquidity(1, 1000)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();


    // Check balances within swap contract:
    // get balances of swap contract
    let response = _swap_contract_instance
        .methods()
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract Native token balance: {:?}", response.value);

    let response = _swap_contract_instance
        .methods()
        .get_balance(alt_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract Alt token balance: {:?}", response.value);
    
    // Logging the token reserves after the add_liquidity
    println!("Swap contract After add_liquidity: {:?}", log.get_logs());



    // Check LP tokens amount to be = 50000
    let wallet_lp_balance: u64 = _wallet.get_asset_balance(&lp_token_id).await.unwrap();
    println!("Wallet1 LP token balance = {}", wallet_lp_balance.clone()); 
    assert_eq!( wallet_lp_balance, <u64 as Into<u64>>::into(50000u64) );

    // Inspect the wallet for alt tokens
    let _wallet_alt_bal = _wallet.get_asset_balance(&alt_token_id).await.unwrap();
    println!("wallet1 Alt token balance : {:?}", _wallet_alt_bal.clone());
    assert_eq!(_wallet_alt_bal, <u64 as Into<u64>>::into(450000u64) );


    //---------------------------------------------------
    // ADDING LIQUIDITY SECOND TIME:

    // Transfer the rest of the alt tokens to the wallet
    let alt_to_wal_amnt: u64 = 500000;
    let _t = _token_contract_instance
        .methods()
        .transfer_coins_to_output(
            alt_to_wal_amnt, 
            _token_contract_id.clone(), 
            Address::from(_wallet.address())
        )
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check the balance of the contract of its own asset, should be zero as we have tx'd all token out.
    let result = _token_contract_instance
        .methods()
        .get_balance(_token_contract_id.clone(), _token_contract_id.clone())
        .call()
        .await
        .unwrap();
    println!("get alt token balance of contract = {}", result.value);      
    assert_eq!(result.value, <u64 as Into<u64>>::into(0u64));


    println!(" ");
    println!("----------------------------------------");  
    println!("After adding liquidity once, and before a second time:"); 
    println!("wallet1 Native balance    = {}", _wallet.get_asset_balance(&BASE_ASSET_ID).await.unwrap()); 
    println!("wallet1 Alt token balance = {}", _wallet.get_asset_balance(&alt_token_id).await.unwrap());
    println!("wallet1 LP token balance  = {}", _wallet.get_asset_balance(&lp_token_id).await.unwrap()); 
    println!("----------------------------------------"); 
    println!(" ");

    println!("Deposit 950000 native assets into the Exchange contract.");
    // Deposit 950000 native assets into the Exchange contract.
    let log = _swap_contract_instance
        .methods()
        .deposit()
        .call_params(
            CallParameters::new(Some(950000), 
            Some(BASE_ASSET_ID), 
            None)
        )
        .call()
        .await
        .unwrap();

    println!("deposit 950000 alt tokens into the Exchange contract.");
    // deposit 950000 alt tokens into the Exchange contract
    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(CallParameters::new(
            Some(950000), 
            Some(alt_token_id.clone()), 
            None)
        )
        .call()
        .await
        .unwrap();
    
    let response = _swap_contract_instance
        .methods()
        .get_balance(alt_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract, Alt token balance: {:?}", response.value);

    println!(" ");
    println!("Add more liquidity 1:1 get back LP tokens:");
    // Add initial liquidity, setting proportion 1:1
    // where lp tokens returned should be equal to the eth_amount deposited (50000 in this case)
    _swap_contract_instance
        .methods()
        .add_liquidity(1, 1000)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();


    // Check balances within swap contract:
    let response = _swap_contract_instance
        .methods()
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract Native token balance: {:?}", response.value);

    let response = _swap_contract_instance
        .methods()
        .get_balance(alt_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract Alt token balance: {:?}", response.value);
    
    // Logging the token reserves after the add_liquidity
    println!("Swap contract After add_liquidity: {:?}", log.get_logs());


    // Check LP tokens amount to be = 950000 + 50000 = 1000000
    let wallet_lp_balance2: u64 = _wallet.get_asset_balance(&lp_token_id).await.unwrap();
    assert_eq!( wallet_lp_balance2, <u64 as Into<u64>>::into(1000000u64) );

    // Inspect the wallet for alt tokens
    let _wallet_alt_bal2 = _wallet.get_asset_balance(&alt_token_id).await.unwrap();
    assert_eq!(_wallet_alt_bal2, <u64 as Into<u64>>::into(0u64) );

    println!(" ");
    println!("----------------------------------------");  
    println!("After adding liquidity a second time:"); 
    println!("wallet1 Native balance    = {}", _wallet.get_asset_balance(&BASE_ASSET_ID).await.unwrap()); 
    println!("wallet1 Alt token balance = {}", _wallet.get_asset_balance(&alt_token_id).await.unwrap());
    println!("wallet1 LP token balance  = {}", _wallet.get_asset_balance(&lp_token_id).await.unwrap()); 
    println!("----------------------------------------"); 
    println!(" ");


}





//-----------------------------------------------------------------

#[tokio::test]
async fn use_liquidity() {
    println!("TEST: use_liquidity()");

    let (
        _swap_contract_instance, 
        _swap_contract_instance2,
        _swap_contract_id, 
        _token_contract_instance, 
        _token_contract_id,
        _wallet,
        _wallet2,
    ) = setup_system_for_test(1000000, 500000).await;

    //--------------------------------------------------
    let alt_token_id = AssetId::from(*_token_contract_id.clone());
    let lp_token_id = AssetId::from(*_swap_contract_id.clone());
    let native_token_id = AssetId::from(*BASE_ASSET_ID);
    // Native asset id
    let native_asset_id = ContractId::new(*BASE_ASSET_ID);
    // Alt asset id
    let alt_asset_id = ContractId::new(*alt_token_id);
    
    println!("alt_token_id    = {}", Address::from(*alt_token_id.clone())); 
    println!("lp_token_id     = {}", Address::from(*lp_token_id.clone())); 
    println!("native_token_id = {}", Address::from(*native_token_id.clone())); 
    println!("native_asset_id = {}", Address::from(*native_asset_id.clone())); 
    println!("alt_asset_id    = {}", Address::from(*alt_asset_id.clone())); 


    println!("Deposit 50000 native assets into the Exchange contract.");
    // Deposit 50000 native assets into the Exchange contract.
    let log = _swap_contract_instance
        .methods()
        .deposit()
        .call_params(
            CallParameters::new(Some(50000), 
            Some(BASE_ASSET_ID), 
            None)
        )
        .call()
        .await
        .unwrap();

    let response = _swap_contract_instance
        .methods()
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract, Native token balance: {:?}", response.value);

    println!("deposit 50000 alt tokens into the Exchange contract.");
    // deposit 50000 alt tokens into the Exchange contract
    _swap_contract_instance
        .methods()
        .deposit()
        .call_params(CallParameters::new(
            Some(50000), 
            Some(alt_token_id.clone()), 
            None)
        )
        .call()
        .await
        .unwrap();
    
    let response = _swap_contract_instance
        .methods()
        .get_balance(alt_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract, Alt token balance: {:?}", response.value);

    println!(" ");
    println!("Add liquidity 1:1 get back LP tokens:");
    // Add initial liquidity, setting proportion 1:1
    // where lp tokens returned should be equal to the eth_amount deposited (50000 in this case)
    _swap_contract_instance
        .methods()
        .add_liquidity(1, 1000)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    // Check balances within swap contract:
    // get balances of swap contract
    let response = _swap_contract_instance
        .methods()
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract Native token balance: {:?}", response.value);

    let response = _swap_contract_instance
        .methods()
        .get_balance(alt_asset_id)
        .call()
        .await
        .unwrap();
    println!("Swap contract Alt token balance: {:?}", response.value);
    
    // Logging the token reserves after the add_liquidity
    println!("Swap contract After add_liquidity: {:?}", log.get_logs());

    // Check LP tokens amount to be = 50000
    let wallet_lp_balance: u64 = _wallet.get_asset_balance(&lp_token_id).await.unwrap();
    println!("Wallet1 LP token balance = {}", wallet_lp_balance.clone()); 
    assert_eq!( wallet_lp_balance, <u64 as Into<u64>>::into(50000u64) );

    // Inspect the wallet for alt tokens
    let _wallet_alt_bal = _wallet.get_asset_balance(&alt_token_id).await.unwrap();
    println!("wallet1 Alt token balance : {:?}", _wallet_alt_bal.clone());
    assert_eq!(_wallet_alt_bal, <u64 as Into<u64>>::into(450000u64) );

    //---------------------------------------------------
    // get the current reserves in the swap contract:

    let eth_id_b256 = Bits256::from_hex_str("0x0000000000000000000000000000000000000000000000000000000000000000").expect("failed");
    let token_id_b256 = Bits256::from_hex_str("0x0bc39f04a606593e96ed7e37fdc3a35e7a05e05eb6cea6ca6b53f0b17d5ba7a4").expect("failed");

    println!(""); 
    println!("----------------------------------------"); 

    let response = _swap_contract_instance
        .methods()
        .test_current_reserve(eth_id_b256)
        .call()
        .await
        .unwrap();
    println!("Swap contract reserves, Native token = {:?}", response.value);

    let response = _swap_contract_instance
        .methods()
        .test_current_reserve(token_id_b256)
        .call()
        .await
        .unwrap();
    println!("Swap contract reserves, Alt token    = {:?}", response.value);

    //---------------------------------------------------
    // attempt to swap native token for lp token, using provided liquidity.
    println!("wallet2.address bech32: {}", _wallet2.address().to_string());
    println!("wallet2 address 0x    : {}", Address::from(_wallet2.address()));

    println!("Wallet2, send in 10000 native assets into the Exchange contract for swap.");
    // Deposit 50000 native assets into the Exchange contract.
    let use_liq = _swap_contract_instance2
        .methods()
        .swap_using_liquidity(20, native_asset_id, alt_asset_id)
        .call_params(
            CallParameters::new(Some(10000), 
            Some(BASE_ASSET_ID), 
            None)
        )
        .call()
        .await
        .unwrap();

    println!("Amount, sender_addr, new_reserve_x, new_reserve_y,  AFTER the swap: {:?}", use_liq.get_logs() );

    print_wallet_balances( _wallet.clone(), _wallet2.clone(), native_token_id, alt_token_id, lp_token_id ).await;

    let response_mm = _swap_contract_instance2
        .methods()
        .get_balance(alt_asset_id)
        .call()
        .await
        .unwrap(); 

    _swap_contract_instance2
        .methods()
        .withdraw(response_mm.value, alt_asset_id.clone())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();


    let response_a2 = _swap_contract_instance2
        .methods()
        .get_balance(native_asset_id)
        .call()
        .await
        .unwrap();
    let response_b2 = _swap_contract_instance2
        .methods()
        .get_balance(alt_asset_id)
        .call()
        .await
        .unwrap();       

    println!("----------------------------------------");  
    println!(" ");

    let response = _swap_contract_instance
        .methods()
        .test_current_reserve(eth_id_b256)
        .call()
        .await
        .unwrap();
    println!("Swap contract reserves, Native token = {:?}", response.value);

    let response = _swap_contract_instance
        .methods()
        .test_current_reserve(token_id_b256)
        .call()
        .await
        .unwrap();
    println!("Swap contract reserves, Alt token    = {:?}", response.value);


    println!(" ");
    println!("Swap contract, Wallet2 Native token balance = {:?}", response_a2.value);
    println!("Swap contract, Wallet2 Alt token balance    = {:?}", response_b2.value);


    print_wallet_balances( _wallet.clone(), _wallet2.clone(), native_token_id, alt_token_id, lp_token_id ).await;


    //------------------------------------------------------------
    // Wallet1 remove liquidity:
    // Remove 60000 native and 40000 alt tokens 
    let remove_liq_amount = 50000;
    let call_params_liq_remove = CallParameters::new(Some(remove_liq_amount), Some(lp_token_id.clone()), None);

    let result = _swap_contract_instance
        .methods()
        .remove_liquidity(60000, 40000, 1000)
        .call_params(call_params_liq_remove)
        .append_variable_outputs(2)
        .call()
        .await
        .unwrap();

    ma::assert_ge!(result.value.eth_amount, 60000);
    ma::assert_ge!(result.value.token_amount, 40000);

    println!("----------------------------------------");  
    println!("remove_liquidity result.value.eth_amount   = {}", result.value.eth_amount); 
    println!("remove_liquidity result.value.token_amount = {}", result.value.token_amount); 
    
    
    // Inspect the wallet for LP tokens should equal 0
    assert_eq!( _wallet.get_asset_balance(&lp_token_id).await.unwrap(), <u64 as Into<u64>>::into(0u64) );

    println!(" ");
    println!("wallet1 Native balance    = {}", _wallet.get_asset_balance(&BASE_ASSET_ID).await.unwrap()); 
    println!("wallet1 Alt token balance = {}", _wallet.get_asset_balance(&alt_token_id).await.unwrap());
    println!("wallet1 LP token balance  = {}", _wallet.get_asset_balance(&lp_token_id).await.unwrap()); 
    println!(" ");
    println!("----------------------------------------"); 
    println!(" ");

    let sw_response_a = _swap_contract_instance
        .methods()
        .test_current_reserve(eth_id_b256)
        .call()
        .await
        .unwrap();
    println!("Swap contract reserves, Native token = {:?}", sw_response_a.value);

    let sw_response_b = _swap_contract_instance
        .methods()
        .test_current_reserve(token_id_b256)
        .call()
        .await
        .unwrap();
    println!("Swap contract reserves, Alt token    = {:?}", sw_response_b.value);

    // check swap contract reserves are now empty
    assert_eq!( sw_response_a.value, <u64 as Into<u64>>::into(0u64) );
    assert_eq!( sw_response_b.value, <u64 as Into<u64>>::into(0u64) );

    // Wallet1:
    //  add     50000 ETH
    //  add     50000 Alt
    //  rec     50000 LP
    // SwapContract:
    //  ETH     50000 ETH
    //  Alt     50000 Alt
    // Wallet2:
    //  swap    10000 ETH
    //  rec      9996 Alt
    // SwapContract:
    //  ETH     60000 ETH
    //  Alt     40004 Alt
    // Wallet1:
    //  dep     50000 LP at 60/40 (min)
    //  rec     60000 ETH
    //  rec     40004 Alt
    // SwapContract:
    //  ETH         0 ETH
    //  Alt         0 Alt


    println!("\n\n");
}



async fn print_wallet_balances(
    _wallet1: WalletUnlocked, 
    _wallet2: WalletUnlocked, 
    _native_token_id: AssetId, 
    _alt_token_id: AssetId,
    _lp_token_id: AssetId,
){
    println!("----------------------------------------");   
    println!(" ");
    println!("wallet1 Native balance    = {}", _wallet1.get_asset_balance(&_native_token_id).await.unwrap()); 
    println!("wallet1 Alt token balance = {}", _wallet1.get_asset_balance(&_alt_token_id).await.unwrap());
    println!("wallet1 LP token balance  = {}", _wallet1.get_asset_balance(&_lp_token_id).await.unwrap()); 
    println!(" ");
    println!("wallet2 Native balance    = {}", _wallet2.get_asset_balance(&_native_token_id).await.unwrap()); 
    println!("wallet2 alt token balance = {}", _wallet2.get_asset_balance(&_alt_token_id).await.unwrap());
    println!("wallet2 LP token balance  = {}", _wallet2.get_asset_balance(&_lp_token_id).await.unwrap()); 
    println!("----------------------------------------");   

}

