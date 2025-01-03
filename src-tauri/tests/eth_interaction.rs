#![allow(dead_code)]

#[cfg(test)]
mod eth_interaction {

    use std::str::FromStr;

    use alloy::{
        network::EthereumWallet,
        primitives::{Address, Bytes, U256},
        providers::ProviderBuilder,
        signers::local::PrivateKeySigner,
        sol,
    };
    use eyre::Result;
    use Chat::Message;

    static ALICE_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    static ALICE_PRIVATE_KEY: &str =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    static BOB_ADDRESS: &str = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";

    static BOB_PRIVATE_KEY: &str =
        "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

    static ANVIL_ENDPOINT: &str = "http://127.0.0.1:8545";

    sol!(
        #[allow(missing_docs)]
        #[sol(rpc)]
        CHAT,
        "../contracts/out/Chat.sol/Chat.json"
    );

    #[tokio::test]
    pub async fn eth_interaction_basics_wotk() -> Result<()> {
        let alice_signer: PrivateKeySigner = ALICE_PRIVATE_KEY.parse().expect("invalid key");
        let alice_wallet = EthereumWallet::from(alice_signer.clone());

        let bob_signer: PrivateKeySigner = BOB_PRIVATE_KEY.parse().expect("invalid key");
        let bob_wallet = EthereumWallet::from(bob_signer.clone());

        // Create two users, Alice and Bob.
        let alice = Address::from_str(ALICE_ADDRESS).expect("invalid address");
        let bob = Address::from_str(BOB_ADDRESS).expect("invalid address");

        // Set up the HTTP provider with the `reqwest` crate.
        let apr = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(alice_wallet.clone());
        let alice_provider = apr.on_http(ANVIL_ENDPOINT.parse().expect("invalid rpc"));

        let bpr = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(bob_wallet.clone());
        let bob_provider = bpr.on_http(ANVIL_ENDPOINT.parse().expect("invalid rpc"));

        let contract = CHAT::deploy(&alice_provider).await.unwrap();
        let alice_contract = CHAT::new(*contract.address(), alice_provider.clone());
        let bob_contract = CHAT::new(*contract.address(), bob_provider.clone());

        let dh_pub_keys = contract.getDHPublicKeys(alice, bob).call().await.unwrap();

        assert!(!dh_pub_keys.ok);

        let msgbytes = Bytes::copy_from_slice(&[1, 2, 3]);
        let tx = alice_contract
            .addFriend(bob, msgbytes.clone())
            .send()
            .await
            .unwrap();
        tx.watch().await.unwrap();

        let tx = bob_contract
            .addFriend(alice, msgbytes.clone())
            .send()
            .await
            .unwrap();
        tx.watch().await.unwrap();

        let dh_pub_keys = contract.getDHPublicKeys(alice, bob).call().await.unwrap();

        assert!(dh_pub_keys.ok);

        let tx = alice_contract
            .sendMessage(
                bob,
                Message {
                    who: Address::ZERO,
                    when: U256::ZERO,
                    encryptedMessage: msgbytes.clone(),
                    nonce: msgbytes.clone(),
                    isFile: false,
                },
            )
            .send()
            .await
            .unwrap();
        tx.watch().await.unwrap();

        let tx = bob_contract
            .sendMessage(
                alice,
                Message {
                    who: Address::ZERO,
                    when: U256::ZERO,
                    encryptedMessage: msgbytes.clone(),
                    nonce: msgbytes.clone(),
                    isFile: false,
                },
            )
            .send()
            .await
            .unwrap();
        tx.watch().await.unwrap();

        let messages = alice_contract
            .getLatestMessages(alice, bob, U256::from(0), U256::from(0))
            .call()
            .await
            .unwrap()
            ._0;

        assert_eq!(messages.len(), 2);

        let friends = alice_contract.getFriendList(alice).call().await.unwrap()._0;
        assert_eq!(friends.len(), 1);

        let alice_contract = CHAT::new(*contract.address(), alice_provider.clone());
        let rrr = alice_contract.whoAmI().call().await.unwrap()._0;

        // NOTE: This is a bug in Alloy probably where msg.sender for call functions is 0x000000
        // (so it's not signing the message looks like)

        Ok(())
    }
}
