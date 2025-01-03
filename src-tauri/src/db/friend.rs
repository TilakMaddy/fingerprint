use std::io::Read;

use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore, Aes256Gcm,
};
use serde::{Deserialize, Serialize};
use x25519_dalek::{SharedSecret, StaticSecret};

use crate::config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Friend {
    pub name: String,
    pub public_address: String,
}

pub fn select_friends(username: String) -> Result<Vec<Friend>, ()> {
    let config = config::config();

    let mut stmt = config
        .conn
        .prepare("SELECT public_address, name FROM Friend where username = :u")
        .unwrap();

    let person_iter_result = stmt.query_map(&[(":u", &username)], |row| {
        Ok(Friend {
            public_address: row.get(0)?,
            name: row.get(1)?,
        })
    });

    match person_iter_result {
        Ok(person_iter) => {
            let frineds = person_iter.flatten().collect();
            return Ok(frineds);
        }
        Err(err) => {
            println!("{:?}", err);
        }
    }

    Ok(vec![])
}

pub fn create(name: String, address: String, cipher: Aes256Gcm, username: String) {
    let config = config::config();

    let exec = config
        .conn
        .execute(
            "
            insert into
              `Friend` (`name`, `public_address`, `username`)
            values
              ( ?, ?, ?);
            ",
            (name, address, username.clone()),
        )
        .unwrap();

    debug_assert!(exec == 1);

    let friend_id = config.conn.last_insert_rowid();

    let my_secret_plaintext = StaticSecret::random();
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let encrypted_my_secret = cipher
        .encrypt(&nonce, my_secret_plaintext.as_ref())
        .unwrap();

    let exec = config
        .conn
        .execute(
            "
            insert into
              `Keys` (
                `enc_my_secret`,
                `my_secret_nonce`,
                `friend_id`,
                `username`
              )
            values
              (?, ?, ?, ?);
        ",
            (encrypted_my_secret, nonce.to_vec(), friend_id, username),
        )
        .unwrap();

    debug_assert!(exec == 1);
}

pub fn store_shared_secret(
    name: String,
    address: String,
    shared_secret: SharedSecret,
    cipher: Aes256Gcm,
    username: String,
) -> Option<()> {
    let config = config::config();
    let friend_id: usize = config
        .conn
        .query_row(
            "select `id` from `Friend` where name = ? and public_address = ? and username = ?;",
            [name, address, username.clone()],
            |row| row.get(0),
        )
        .ok()?;

    let shared_secret_nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let enc_shared_secret = cipher
        .encrypt(&shared_secret_nonce, shared_secret.as_ref())
        .unwrap();

    let exec = config
        .conn
        .execute(
            "
    update Keys 
    set 
        `enc_shared_secret` = ?,
       `shared_secret_nonce` = ?
    where
        `friend_id` = ? and
        `username` = ?
        ",
            (
                enc_shared_secret,
                shared_secret_nonce.to_vec(),
                friend_id,
                username,
            ),
        )
        .unwrap();

    debug_assert!(exec == 1);
    Some(())
}

pub fn friend_exists(name: String, address: String, username: String) -> bool {
    let config = config::config();
    config
        .conn
        .query_row(
            "select `id` from `Friend` where name = ? and public_address = ? and username = ?;",
            [name, address, username],
            |row| row.get::<_, usize>(0),
        )
        .is_ok()
}

pub fn shared_secret(
    name: String,
    address: String,
    username: String,
) -> Option<(Vec<u8>, Vec<u8>)> {
    let config = config::config();
    let friend_id: isize = config
        .conn
        .query_row(
            "select `id` from `Friend` where name = ? and public_address = ? and username = ?;",
            [name, address, username.clone()],
            |row| row.get(0),
        )
        .ok()?;

    let row_id: i64 = config
        .conn
        .query_row(
            "select `id` from `Keys` where friend_id = ? and username = ?;",
            (friend_id, username.clone()),
            |row| row.get(0),
        )
        .ok()?;

    let enc_shared_secret = config
        .conn
        .blob_open(
            rusqlite::DatabaseName::Main,
            "Keys",
            "enc_shared_secret",
            row_id,
            true,
        )
        .ok()?;

    let shared_secret_nonce = config
        .conn
        .blob_open(
            rusqlite::DatabaseName::Main,
            "Keys",
            "shared_secret_nonce",
            row_id,
            true,
        )
        .ok()?;

    Some((
        enc_shared_secret.bytes().flatten().collect::<Vec<_>>(),
        shared_secret_nonce.bytes().flatten().collect::<Vec<_>>(),
    ))
}

pub fn my_secret(name: String, address: String, username: String) -> Option<(Vec<u8>, Vec<u8>)> {
    let config = config::config();
    let friend_id: usize = config
        .conn
        .query_row(
            "select `id` from `Friend` where name = ? and public_address = ? and username = ?;",
            [name, address, username.clone()],
            |row| row.get(0),
        )
        .ok()?;

    let row_id: i64 = config
        .conn
        .query_row(
            "select `id` from `Keys` where friend_id = ? and username = ?;",
            (friend_id, username),
            |row| row.get(0),
        )
        .ok()?;

    let enc_my_secret = config
        .conn
        .blob_open(
            rusqlite::DatabaseName::Main,
            "Keys",
            "enc_my_secret",
            row_id,
            true,
        )
        .ok()?;

    let my_secret_nonce = config
        .conn
        .blob_open(
            rusqlite::DatabaseName::Main,
            "Keys",
            "my_secret_nonce",
            row_id,
            true,
        )
        .ok()?;

    Some((
        enc_my_secret.bytes().flatten().collect::<Vec<_>>(),
        my_secret_nonce.bytes().flatten().collect::<Vec<_>>(),
    ))
}
