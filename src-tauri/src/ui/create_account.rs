use tokio::sync::Mutex;

use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use argon2::{password_hash::SaltString, Argon2};
use md5::{Digest, Md5};
use serde::Deserialize;
use tauri::State;

use crate::{db, LocalEncryptionCipher};

// Frontend DTO
#[derive(Debug, Deserialize)]
pub struct UserSubmittedInfo {
    pub username: String,
    pub password: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
}

#[tauri::command]
pub async fn create_account(
    state: State<'_, Mutex<LocalEncryptionCipher>>,
    payload: UserSubmittedInfo,
) -> Result<(), ()> {
    // Extract fields to insert into the database

    // username
    let username = payload.username.clone();

    // password-hash
    let mut hasher = Md5::new();
    hasher.update(payload.password.clone().as_bytes());
    let password_hash = hasher.finalize();

    // password_salt
    let salt = SaltString::generate(&mut OsRng);
    let mut salt_bytes = [0u8; 16];
    let password_salt = salt
        .decode_b64(&mut salt_bytes)
        .expect("decoding doesnt work");

    // private_key_nonce
    let private_key_nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message

    // enc_private_key
    let mut argon_output_key = [0u8; 32]; // 256 bit key
    Argon2::default()
        .hash_password_into(
            payload.password.clone().as_bytes(),
            password_salt,
            &mut argon_output_key,
        )
        .expect("argon hashing failed");
    let aes_key: Key<Aes256Gcm> = argon_output_key.into();
    let cipher = Aes256Gcm::new(&aes_key);
    let enc_private_key = cipher
        .encrypt(&private_key_nonce, payload.private_key.clone().as_bytes())
        .unwrap();

    // Insert into database
    db::personal::create(
        username.clone(),
        String::from_utf8_lossy(&password_hash).to_string(),
        password_salt,
        &private_key_nonce,
        &enc_private_key,
    );

    // Set the cipher as state so that the whole application can get access to the cipher
    let mut state_lock = state.lock().await;
    state_lock.cipher.replace(cipher.clone());
    state_lock
        .private_key
        .replace(db::personal::private_key(username.clone(), cipher));
    state_lock.username.replace(username);

    Ok(())
}
