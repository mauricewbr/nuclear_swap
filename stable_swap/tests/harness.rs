use fuels::prelude::*;
use fuels::tx::{AssetId, ContractId, Salt};
use fuels_abigen_macro::abigen;

abigen!(NuclearSwap, "out/debug/stable_swap-abi.json");
abigen!(Asset, "tests/artifacts/asset/out/debug/asset-abi.json");

struct Metadata {
    nuclear: NuclearSwap,
    asset: Option<Asset>,
    wallet: LocalWallet,
}

async fn setup() -> (Metadata, Metadata, Metadata, ContractId, u64) {
    let num_wallets = 3;
    let coins_per_wallet = 1;
    let amount_per_coin = 1_000_000;

    let config = WalletsConfig::new(
        Some(num_wallets),
        Some(coins_per_wallet),
        Some(amount_per_coin),
    );

    let mut wallets = launch_provider_and_get_wallets(config).await;

    let deployer_wallet = wallets.pop().unwrap();
    let lp_wallet = wallets.pop().unwrap();
    let swapper_wallet = wallets.pop().unwrap();

    let nuclearswap_id = Contract::deploy(
        "./out/debug/stable_swap.bin",
        &deployer_wallet,
        TxParameters::default(),
    )
    .await
    .unwrap();

    let asset_id = Contract::deploy(
        "./tests/artifacts/asset/out/debug/asset.bin",
        &deployer_wallet,
        TxParameters::default(),
    )
    .await
    .unwrap();

    let deployer = Metadata {
        nuclear: NuclearSwap::new(nuclearswap_id.to_string(), deployer_wallet.clone()),
        asset: Some(Asset::new(asset_id.to_string(), deployer_wallet.clone())),
        wallet: deployer_wallet,
    };

    let lp = Metadata {
        nuclear: NuclearSwap::new(nuclearswap_id.to_string(), lp_wallet.clone()),
        asset: None,
        wallet: lp_wallet,
    };

    let swapper = Metadata {
        nuclear: NuclearSwap::new(nuclearswap_id.to_string(), swapper_wallet.clone()),
        asset: None,
        wallet: swapper_wallet,
    };

    let asset_amount: u64 = 100;

    (deployer, lp, swapper, asset_id, asset_amount)
}
/*
async fn init(
    deployer: &Metadata,
    lp: &LocalWallet,
    swapper: &LocalWallet,
    asset_id: ContractId,
    asset_amount: u64,
) -> bool {
    deployer
        .nuclear
        .constructor(lp.address(), swapper.address(), asset_id, asset_amount)
        .call()
        .await
        .unwrap()
        .value
}

async fn mint(deployer: &Metadata, user: &LocalWallet, asset_amount: u64) {
    deployer
        .asset
        .as_ref()
        .unwrap()
        .mint_and_send_to_address(asset_amount, user.address())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap()
        .value;
}

async fn balance(nuclear: &NuclearSwap) -> u64 {
    nuclear.get_balance().call().await.unwrap().value
}

async fn user_data(nuclear: &NuclearSwap, user: &LocalWallet) -> (bool, bool) {
    nuclear
        .get_user_data(user.address())
        .call()
        .await
        .unwrap()
        .value
}


mod constructor {

    use super::*;

    #[tokio::test]
    async fn initializes() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        assert!(
            init(
                &deployer,
                &lp.wallet,
                &swapper.wallet,
                asset_id,
                asset_amount
            )
            .await
        );
    }
}

mod deposit {

    use super::*;

    #[tokio::test]
    async fn deposits() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
        let call_params = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &lp.wallet, asset_amount).await;

        assert_eq!(balance(&deployer.nuclear).await, 0);
        assert_eq!(
            user_data(&deployer.nuclear, &lp.wallet).await,
            (false, false)
        );

        // Test
        assert!(
            lp
                .nuclear
                .deposit()
                .tx_params(tx_params)
                .call_params(call_params)
                .call()
                .await
                .unwrap()
                .value
        );

        assert_eq!(balance(&deployer.nuclear).await, asset_amount);
        assert_eq!(
            user_data(&deployer.nuclear, &lp.wallet).await,
            (true, false)
        );
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_initialized() {
        let (_, lp, _, _, _) = setup().await;

        // Should panic
        lp.nuclear.deposit().call().await.unwrap();
    }

    // Uncomment when https://github.com/FuelLabs/fuels-rs/pull/305 (deploy_with_salt) lands in a new release
    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_with_incorrect_asset() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let another_asset_id = Contract::deploy_with_salt(
            "./tests/artifacts/asset/out/debug/asset.bin",
            &deployer.wallet,
            TxParameters::default(),
            Salt::from([1u8; 32]),
        )
        .await
        .unwrap();

        let another_asset = Asset::new(another_asset_id.to_string(), deployer.wallet.clone());

        let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
        let call_params =
            CallParameters::new(Some(asset_amount), Some(AssetId::from(*another_asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        another_asset
            .mint_and_send_to_address(asset_amount, lp.wallet.address())
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap();

        // Should panic
        lp
            .nuclear
            .deposit()
            .tx_params(tx_params)
            .call_params(call_params)
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_with_incorrect_asset_amount() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
        let call_params =
            CallParameters::new(Some(asset_amount - 1), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &lp.wallet, asset_amount).await;

        // Should panic
        lp
            .nuclear
            .deposit()
            .tx_params(tx_params)
            .call_params(call_params)
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_sender_is_not_the_correct_address() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
        let call_params = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &deployer.wallet, asset_amount).await;

        // Should panic
        deployer
            .nuclear
            .deposit()
            .tx_params(tx_params)
            .call_params(call_params)
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_already_deposited() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &lp.wallet, 2 * asset_amount).await;

        lp
            .nuclear
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();

        // Should panic
        lp
            .nuclear
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_after_both_parties_approve() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params3 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params3 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &lp.wallet, asset_amount).await;
        mint(&deployer, &swapper.wallet, asset_amount).await;

        lp
            .nuclear
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        swapper
            .nuclear
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        lp.nuclear.approve().call().await.unwrap();
        swapper
            .nuclear
            .approve()
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap();

        // Should panic
        lp
            .nuclear
            .deposit()
            .tx_params(tx_params3)
            .call_params(call_params3)
            .call()
            .await
            .unwrap();
    }
}

mod approve {

    use super::*;

    #[tokio::test]
    async fn approves() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &lp.wallet, asset_amount).await;
        mint(&deployer, &swapper.wallet, asset_amount).await;

        lp
            .nuclear
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        swapper
            .nuclear
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        assert_eq!(
            user_data(&deployer.nuclear, &lp.wallet).await,
            (true, false)
        );
        assert_eq!(
            user_data(&deployer.nuclear, &swapper.wallet).await,
            (true, false)
        );
        assert_eq!(balance(&deployer.nuclear).await, 2 * asset_amount);

        // Test
        assert!(lp.nuclear.approve().call().await.unwrap().value);
        assert!(
            swapper
                .nuclear
                .approve()
                .append_variable_outputs(2)
                .call()
                .await
                .unwrap()
                .value
        );

        assert_eq!(
            user_data(&deployer.nuclear, &lp.wallet).await,
            (true, true)
        );
        assert_eq!(
            user_data(&deployer.nuclear, &swapper.wallet).await,
            (true, true)
        );
        assert_eq!(balance(&deployer.nuclear).await, 0);
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_initialized() {
        let (_, lp, _, _, _) = setup().await;

        // Should panic
        lp.nuclear.approve().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_sender_is_not_the_correct_address() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        // Should panic
        deployer.nuclear.approve().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_deposited() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        // Should panic
        lp.nuclear.approve().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_after_both_parties_approve() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &lp.wallet, asset_amount).await;
        mint(&deployer, &swapper.wallet, asset_amount).await;

        lp
            .nuclear
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        swapper
            .nuclear
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        lp.nuclear.approve().call().await.unwrap();
        swapper
            .nuclear
            .approve()
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap();

        // Should panic
        lp.nuclear.approve().call().await.unwrap();
    }
}

mod withdraw {

    use super::*;

    #[tokio::test]
    async fn withdraws() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
        let call_params = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &lp.wallet, asset_amount).await;

        lp
            .nuclear
            .deposit()
            .tx_params(tx_params)
            .call_params(call_params)
            .call()
            .await
            .unwrap();

        lp.nuclear.approve().call().await.unwrap();

        assert_eq!(
            user_data(&deployer.nuclear, &lp.wallet).await,
            (true, true)
        );
        assert_eq!(balance(&deployer.nuclear).await, asset_amount);

        // Test
        assert!(
            lp
                .nuclear
                .withdraw()
                .append_variable_outputs(1)
                .call()
                .await
                .unwrap()
                .value
        );

        assert_eq!(
            user_data(&deployer.nuclear, &lp.wallet).await,
            (false, false)
        );
        assert_eq!(balance(&deployer.nuclear).await, 0);
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_initialized() {
        let (_, lp, _, _, _) = setup().await;

        // Should panic
        lp.nuclear.withdraw().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_sender_is_not_the_correct_address() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        // Should panic
        deployer.nuclear.withdraw().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_deposited() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        // Should panic
        lp.nuclear.withdraw().call().await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_after_both_parties_approve() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &lp.wallet, asset_amount).await;
        mint(&deployer, &swapper.wallet, asset_amount).await;

        lp
            .nuclear
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        swapper
            .nuclear
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        lp.nuclear.approve().call().await.unwrap();
        swapper
            .nuclear
            .approve()
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap();

        // Should panic
        lp
            .nuclear
            .withdraw()
            .append_variable_outputs(1)
            .call()
            .await
            .unwrap();
    }
}

mod get_balance {

    use super::*;

    #[tokio::test]
    async fn returns_zero() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        assert_eq!(balance(&deployer.nuclear).await, 0);
    }

    #[tokio::test]
    async fn returns_asset_amount() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params = TxParameters::new(None, Some(1_000_000), None, None);
        let call_params = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &lp.wallet, asset_amount).await;

        lp
            .nuclear
            .deposit()
            .tx_params(tx_params)
            .call_params(call_params)
            .call()
            .await
            .unwrap();

        assert_eq!(balance(&deployer.nuclear).await, asset_amount);
    }
}

mod get_user_data {

    use super::*;

    #[tokio::test]
    async fn gets_user_data() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &lp.wallet, asset_amount).await;
        mint(&deployer, &swapper.wallet, asset_amount).await;

        assert_eq!(
            user_data(&deployer.nuclear, &lp.wallet).await,
            (false, false)
        );
        assert_eq!(
            user_data(&deployer.nuclear, &swapper.wallet).await,
            (false, false)
        );

        lp
            .nuclear
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        swapper
            .nuclear
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        lp.nuclear.approve().call().await.unwrap();
        swapper
            .nuclear
            .approve()
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap();

        assert_eq!(
            user_data(&deployer.nuclear, &lp.wallet).await,
            (true, true)
        );
        assert_eq!(
            user_data(&deployer.nuclear, &swapper.wallet).await,
            (true, true)
        );
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_not_initialized() {
        let (_, lp, _, _, _) = setup().await;

        // Should panic
        lp
            .nuclear
            .get_user_data(lp.wallet.address())
            .call()
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(42)")]
    async fn panics_when_sender_is_not_the_correct_address() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        // Should panic
        lp
            .nuclear
            .get_user_data(deployer.wallet.address())
            .call()
            .await
            .unwrap();
    }
}

mod get_state {

    use super::*;

    #[tokio::test]
    async fn not_initialized() {
        let (deployer, _, _, _, _) = setup().await;

        assert_eq!(deployer.nuclear.get_state().call().await.unwrap().value, 0);
    }

    #[tokio::test]
    async fn initialized() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        // Init conditions
        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;

        assert_eq!(deployer.nuclear.get_state().call().await.unwrap().value, 1);
    }

    #[tokio::test]
    async fn completed() {
        let (deployer, lp, swapper, asset_id, asset_amount) = setup().await;

        let tx_params1 = TxParameters::new(None, Some(1_000_000), None, None);
        let tx_params2 = TxParameters::new(None, Some(1_000_000), None, None);

        let call_params1 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));
        let call_params2 = CallParameters::new(Some(asset_amount), Some(AssetId::from(*asset_id)));

        // Init conditions
        assert_eq!(deployer.nuclear.get_state().call().await.unwrap().value, 0);

        init(
            &deployer,
            &lp.wallet,
            &swapper.wallet,
            asset_id,
            asset_amount,
        )
        .await;
        mint(&deployer, &lp.wallet, asset_amount).await;
        mint(&deployer, &swapper.wallet, asset_amount).await;

        assert_eq!(deployer.nuclear.get_state().call().await.unwrap().value, 1);

        lp
            .nuclear
            .deposit()
            .tx_params(tx_params1)
            .call_params(call_params1)
            .call()
            .await
            .unwrap();
        swapper
            .nuclear
            .deposit()
            .tx_params(tx_params2)
            .call_params(call_params2)
            .call()
            .await
            .unwrap();

        // Test
        lp.nuclear.approve().call().await.unwrap();
        swapper
            .nuclear
            .approve()
            .append_variable_outputs(2)
            .call()
            .await
            .unwrap();

        assert_eq!(deployer.nuclear.get_state().call().await.unwrap().value, 2);
    }
}
*/