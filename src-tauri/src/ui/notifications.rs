use crate::{eth, LocalEncryptionCipher};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

use alloy::signers::local::PrivateKeySigner;

#[tauri::command]
pub async fn get_incoming_friend_requests(app: AppHandle) -> Result<Vec<String>, ()> {
    let lec_state = app.state::<Mutex<LocalEncryptionCipher>>();
    let state_lock = lec_state.lock().await;
    let private_key = state_lock.private_key.clone().unwrap();

    let signer: PrivateKeySigner = private_key.parse().expect("invalid key");

    let incoming_fr = eth::queries::get_incoming_friend_requests(signer.address())
        .await
        .unwrap();

    Ok(incoming_fr
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>())
}

#[tauri::command]
pub async fn get_pending_outgoing_requests(app: AppHandle) -> Result<Vec<String>, ()> {
    let lec_state = app.state::<Mutex<LocalEncryptionCipher>>();
    let state_lock = lec_state.lock().await;
    let private_key = state_lock.private_key.clone().unwrap();

    let signer: PrivateKeySigner = private_key.parse().expect("invalid key");

    let incoming_fr = eth::queries::get_outgoing_friend_requests(signer.address())
        .await
        .unwrap();

    Ok(incoming_fr
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>())
}
