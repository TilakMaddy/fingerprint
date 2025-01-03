use tokio::sync::Mutex;

use crate::{
    db::{self, friend::Friend},
    eth::{self, mutators::DMessage},
    LocalEncryptionCipher,
};
use aes_gcm::aead::{generic_array::GenericArray, Aead};
use tauri::{AppHandle, Manager};
use x25519_dalek::{PublicKey, StaticSecret};

use alloy::signers::local::PrivateKeySigner;

#[tauri::command]
pub async fn state_is_filled(app: AppHandle) -> Result<bool, ()> {
    let lec_state = app.state::<Mutex<LocalEncryptionCipher>>();
    let state_lock = lec_state.lock().await;
    let filled = state_lock.cipher.is_some() && state_lock.private_key.is_some();
    Ok(filled)
}

#[tauri::command]
pub async fn select_friends(app: AppHandle) -> Result<Vec<Friend>, ()> {
    let lec_state = app.state::<Mutex<LocalEncryptionCipher>>();
    let state_lock = lec_state.lock().await;
    let private_key = state_lock.private_key.clone().unwrap();
    let username = state_lock.username.clone().unwrap();

    let signer: PrivateKeySigner = private_key.parse().expect("invalid key");

    let frsss = eth::queries::get_friend_list(signer.address())
        .await
        .unwrap();

    let allsfssaf = db::friend::select_friends(username.clone()).unwrap_or_default();

    let h = allsfssaf
        .into_iter()
        .filter(|x| frsss.iter().any(|y| y.to_string() == x.public_address))
        .collect();

    Ok(h)
    //Ok(frsss.into_iter().map(|x| x.to_string()).collect::<Vec<_>>())
}

#[tauri::command]
pub async fn check_and_update_ssk(app: AppHandle, name: String, friend_address: String) {
    let state = app.state::<Mutex<LocalEncryptionCipher>>();
    let state_lock = state.lock().await;
    let cipher = state_lock.cipher.clone().unwrap();
    let private_key = state_lock.private_key.clone().unwrap();
    let username = state_lock.username.clone().unwrap();

    let signer: PrivateKeySigner = private_key.parse().expect("invalid key");

    // If already shared scret for frend is there, return early
    if let Some((_, _)) =
        db::friend::shared_secret(name.clone(), friend_address.clone(), username.clone())
    {
        return;
    }

    // Calculate the public key to share with our friend
    let Some((enc_my_secret, my_secret_nonce)) =
        db::friend::my_secret(name.clone(), friend_address.clone(), username.clone())
    else {
        return;
    };

    let plain_text = cipher
        .decrypt(
            GenericArray::from_slice(my_secret_nonce.as_slice()),
            enc_my_secret.as_slice(),
        )
        .unwrap();

    let plaintext_secret_bytes = to_fixed_array(plain_text);
    let secret = StaticSecret::from(plaintext_secret_bytes);

    if eth::queries::are_friends(signer.address(), &friend_address).await {
        // Grab the DH key
        if let Some(dh_pub_key) =
            eth::queries::get_dh_pubkey(signer.address(), &friend_address).await
        {
            let dh_pub_key = PublicKey::from(to_fixed_array(dh_pub_key.to_vec()));
            let plaintext_shared_secret = secret.diffie_hellman(&dh_pub_key);

            // Store the shared secret in our database
            let go = db::friend::store_shared_secret(
                name,
                friend_address,
                plaintext_shared_secret,
                cipher,
                username.clone(),
            );
            dbg!(go);
        }
    }
}

#[tauri::command]
pub async fn add_friend(app: AppHandle, name: String, friend_address: String) -> Result<(), ()> {
    let state = app.state::<Mutex<LocalEncryptionCipher>>();
    let state_lock = state.lock().await;
    let cipher = state_lock.cipher.clone().unwrap();
    let private_key = state_lock.private_key.clone().unwrap();
    let username = state_lock.username.clone().unwrap();

    let signer: PrivateKeySigner = private_key.parse().expect("invalid key");

    // If already shared scret for frend is there, return early
    if let Some((_, _)) =
        db::friend::shared_secret(name.clone(), friend_address.clone(), username.clone())
    {
        return Ok(());
    }

    // If friend is not there in DB, create friend
    if !db::friend::friend_exists(name.clone(), friend_address.clone(), username.clone()) {
        db::friend::create(
            name.clone(),
            friend_address.clone(),
            cipher.clone(),
            username.clone(),
        );
    }

    // Calculate the public key to share with our friend
    let (enc_my_secret, my_secret_nonce) =
        db::friend::my_secret(name.clone(), friend_address.clone(), username.clone()).unwrap();

    let plain_text = cipher
        .decrypt(
            GenericArray::from_slice(my_secret_nonce.as_slice()),
            enc_my_secret.as_slice(),
        )
        .unwrap();

    let plaintext_secret_bytes = to_fixed_array(plain_text);
    let secret = StaticSecret::from(plaintext_secret_bytes);
    let my_public_key = PublicKey::from(&secret);

    // Call the add_friend in the smart contract
    eth::mutators::add_friend(&private_key, &friend_address, my_public_key.to_bytes()).await;

    // Then check, if ACCEPTED status reached. If so, grab their
    // DH public key and construct the shared_secret and commit to DB
    if eth::queries::are_friends(signer.address(), &friend_address).await {
        // Grab the DH key
        if let Some(dh_pub_key) =
            eth::queries::get_dh_pubkey(signer.address(), &friend_address).await
        {
            let dh_pub_key = PublicKey::from(to_fixed_array(dh_pub_key.to_vec()));

            let plaintext_shared_secret = secret.diffie_hellman(&dh_pub_key);

            // Store the shared secret in our database
            let go = db::friend::store_shared_secret(
                name,
                friend_address,
                plaintext_shared_secret,
                cipher,
                username.clone(),
            );
            debug_assert!(go.is_some());
        }
    }

    drop(state_lock);
    Ok(())
}

fn to_fixed_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

#[tauri::command]
pub async fn read_messages(
    app: AppHandle,
    friend_address: String,
    friend_name: String,
) -> Result<Vec<DMessage>, ()> {
    let state = app.state::<Mutex<LocalEncryptionCipher>>();
    let state_lock = state.lock().await;
    let cipher = state_lock.cipher.clone().unwrap();
    let private_key = state_lock.private_key.clone().unwrap();
    let username = state_lock.username.clone().unwrap();

    let messages = eth::mutators::read_all_messages(
        cipher,
        &private_key,
        &friend_address,
        friend_name,
        username,
    )
    .await
    .unwrap();

    Ok(messages)
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    friend_address: String,
    friend_name: String,
    message: String,
) -> Result<(), ()> {
    let state = app.state::<Mutex<LocalEncryptionCipher>>();
    let state_lock = state.lock().await;
    let cipher = state_lock.cipher.clone().unwrap();
    let private_key = state_lock.private_key.clone().unwrap();
    let username = state_lock.username.clone().unwrap();

    let snet = eth::mutators::send_message(
        cipher,
        &private_key,
        &friend_address,
        message,
        friend_name,
        username,
    )
    .await;
    dbg!(snet);

    Ok(())
}
