use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{db, LocalEncryptionCipher};

#[derive(Default, Deserialize, Serialize)]
pub struct UserLoginInfo {
    pub username: String,
    pub password: String,
}

#[tauri::command]
pub async fn login(
    state: State<'_, Mutex<LocalEncryptionCipher>>,
    payload: UserLoginInfo,
) -> Result<(), ()> {
    let cipher = db::personal::cipher(payload.username.clone(), payload.password.clone());
    let mut state_lock = state.lock().await;
    state_lock.cipher.replace(cipher.clone());
    state_lock
        .private_key
        .replace(db::personal::private_key(payload.username.clone(), cipher));
    state_lock.username.replace(payload.username);
    Ok(())
}

#[tauri::command]
pub async fn check_if_password_looks_fine(p: UserLoginInfo) -> Result<bool, ()> {
    if db::personal::password_looks_fine(p.username.clone(), p.password.clone()) {
        return Ok(true);
    }
    Ok(false)
}
