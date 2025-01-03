use aes_gcm::{
    aead::{generic_array::GenericArray, Aead},
    Aes256Gcm, Key, KeyInit,
};
use argon2::Argon2;
use md5::{Digest, Md5};
use std::io::Read;

use crate::config;

pub fn create(
    username: String,
    password_hash: String,
    password_salt: &[u8],
    private_key_nonce: &[u8],
    enc_private_key: &[u8],
) {
    let config = config::config();
    let rows = config
        .conn
        .execute(
            "
            insert into
              `Personal` (
                `enc_private_key`,
                `password_hash`,
                `password_salt`,
                `private_key_nonce`,
                `username`
              )
            values
              (?,?,?,?,?);",
            (
                enc_private_key,
                password_hash,
                password_salt,
                private_key_nonce,
                username,
            ),
        )
        .unwrap();
    debug_assert!(rows == 1);
}

pub fn password_looks_fine(username: String, password: String) -> bool {
    let config = config::config();

    let hash: String = config
        .conn
        .query_row(
            "
            select 
                `password_hash`
            from `Personal` where `username` = ?;",
            [username],
            |row| row.get(0),
        )
        .unwrap();

    let mut hasher = Md5::new();
    hasher.update(password.as_bytes());
    let hashed_bytes = hasher.finalize();
    let lossy_string = String::from_utf8_lossy(&hashed_bytes).to_string();

    lossy_string == hash
}

pub fn cipher(username: String, password: String) -> Aes256Gcm {
    let config = config::config();

    let id: i64 = config
        .conn
        .query_row(
            "
            select 
                `id`
            from `Personal` where `username` = ?;",
            [username],
            |row| row.get(0),
        )
        .unwrap();

    // Encrypted zksync private key
    let password_salt = config
        .conn
        .blob_open(
            rusqlite::DatabaseName::Main,
            "Personal",
            "password_salt",
            id,
            true,
        )
        .unwrap();

    let salt_bytes = password_salt.bytes().flatten().collect::<Vec<_>>();

    let mut argon_output_key = [0u8; 32]; // 256 bit key
    Argon2::default()
        .hash_password_into(password.as_bytes(), &salt_bytes, &mut argon_output_key)
        .expect("argon hashing failed");

    let aes_key: Key<Aes256Gcm> = argon_output_key.into();
    Aes256Gcm::new(&aes_key)
}

pub fn private_key(username: String, cipher: Aes256Gcm) -> String {
    let config = config::config();

    let id: i64 = config
        .conn
        .query_row(
            "
            select 
                `id`
            from `Personal` where `username` = ?;",
            [username],
            |row| row.get(0),
        )
        .unwrap();

    // Encrypted zksync private key
    let enc_pk = config
        .conn
        .blob_open(
            rusqlite::DatabaseName::Main,
            "Personal",
            "enc_private_key",
            id,
            true,
        )
        .unwrap();

    // Nonce
    let nonce_bytes = config
        .conn
        .blob_open(
            rusqlite::DatabaseName::Main,
            "Personal",
            "private_key_nonce",
            id,
            true,
        )
        .unwrap();

    let nonce_bytes = nonce_bytes.bytes().flatten().collect::<Vec<_>>();
    let enc_pk_bytes = enc_pk.bytes().flatten().collect::<Vec<_>>();
    let nonce = GenericArray::from_slice(nonce_bytes.as_slice());

    let private_key_bytes = cipher.decrypt(nonce, enc_pk_bytes.as_slice()).unwrap();
    String::from_utf8(private_key_bytes).unwrap()
}
