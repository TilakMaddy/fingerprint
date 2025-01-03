use std::str::FromStr;

use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use alloy::{
    network::EthereumWallet,
    primitives::{Address, Bytes, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use md5::digest::generic_array::GenericArray;
use serde::{Deserialize, Serialize};

use super::{Chat::Message, CHAT};
use crate::{config, db::friend::shared_secret};

pub async fn add_friend(private_key: &str, friend_address: &str, msgbytes: [u8; 32]) {
    let conf = config::config();
    let fren = Address::from_str(friend_address).expect("invalid fren");

    let signer: PrivateKeySigner = private_key.parse().expect("invalid key");
    let wallet = EthereumWallet::from(signer.clone());

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(conf.rpc.parse().expect("invalid rpc"));

    let contract = CHAT::new(conf.contract_address.parse().unwrap(), provider);

    let msgbytes = Bytes::copy_from_slice(msgbytes.as_slice());

    let tx = contract.addFriend(fren, msgbytes).send().await.unwrap();

    tx.watch().await.unwrap();
}

pub async fn send_message(
    cipher: Aes256Gcm,
    private_key: &str,
    friend_address: &str,
    message: String,
    friend_name: String,
    username: String,
) {
    let conf = config::config();
    let fren = Address::from_str(friend_address).expect("invalid fren");

    let signer: PrivateKeySigner = private_key.parse().expect("invalid key");
    let wallet = EthereumWallet::from(signer.clone());

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(conf.rpc.parse().expect("invalid rpc"));

    let contract = CHAT::new(conf.contract_address.parse().unwrap(), provider);

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let nonce_bytes = Bytes::copy_from_slice(nonce.as_slice());

    // use sscipher instead of normal cipher
    let (enc_ss, ss_nonce) = shared_secret(
        friend_name.clone(),
        friend_address.to_string(),
        username.clone(),
    )
    .unwrap();

    let plain_ss = cipher
        .decrypt(
            GenericArray::from_slice(ss_nonce.to_vec().as_slice()),
            enc_ss.as_ref(),
        )
        .unwrap();

    // Now decrypt the message with the shared secret

    let sskey: Key<Aes256Gcm> = to_fixed_array(plain_ss.clone()).into();
    let sscipher = Aes256Gcm::new(&sskey);

    // END
    let enc_message = sscipher.encrypt(&nonce, message.as_ref());

    let message = Message {
        encryptedMessage: Bytes::from(enc_message.unwrap()),
        nonce: nonce_bytes,
        isFile: false,
        who: Address::ZERO,
        when: U256::ZERO,
    };

    let tx = contract.sendMessage(fren, message).send().await.unwrap();

    tx.watch().await.unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DMessage {
    theirs: bool,
    message: String,
}

pub async fn read_all_messages(
    cipher: Aes256Gcm,
    private_key: &str,
    friend_address: &str,
    friend_name: String,
    username: String,
) -> Result<Vec<DMessage>, ()> {
    let conf = config::config();
    let fren = Address::from_str(friend_address).expect("invalid fren");

    let signer: PrivateKeySigner = private_key.parse().expect("invalid key");
    let wallet = EthereumWallet::from(signer.clone());

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(conf.rpc.parse().expect("invalid rpc"));

    let contract = CHAT::new(conf.contract_address.parse().unwrap(), provider);

    let cryptic_messages = contract
        .getLatestMessages(signer.address(), fren, U256::ZERO, U256::ZERO)
        .call()
        .await
        .unwrap()
        ._0;

    let mut dms = vec![];

    for cm in cryptic_messages {
        let nonce_bytes = cm.nonce.clone();
        let enc_msg_bytes = cm.encryptedMessage.clone();
        let who = cm.who;

        //  this cipher uses password as key instead of shared_secret
        let (enc_ss, ss_nonce) = shared_secret(
            friend_name.clone(),
            friend_address.to_string(),
            username.clone(),
        )
        .unwrap();

        let plain_ss = cipher
            .decrypt(
                GenericArray::from_slice(ss_nonce.to_vec().as_slice()),
                enc_ss.as_ref(),
            )
            .unwrap();

        // Now decrypt the message with the shared secret

        let sskey: Key<Aes256Gcm> = to_fixed_array(plain_ss.clone()).into();
        let sscipher = Aes256Gcm::new(&sskey);

        if let Ok(plaintmsg) = sscipher.decrypt(
            GenericArray::from_slice(nonce_bytes.to_vec().as_slice()),
            enc_msg_bytes.as_ref(),
        ) {
            let ptms = String::from_utf8_lossy(&plaintmsg).to_string();

            dms.push(DMessage {
                theirs: who != signer.address(),
                message: ptms,
            })
        } else {
            println!("SKIPPED MESSAGE!");
            panic!("dfsdfadshufs");
        }
    }

    Ok(dms)
}

fn to_fixed_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}
