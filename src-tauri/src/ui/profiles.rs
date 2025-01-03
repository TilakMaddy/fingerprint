use crate::{config, LocalEncryptionCipher};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

use alloy::{
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
};

#[tauri::command]
pub async fn get_account_0x(app: AppHandle) -> Result<(String, String, String), ()> {
    let lec_state = app.state::<Mutex<LocalEncryptionCipher>>();
    let state_lock = lec_state.lock().await;
    let private_key = state_lock.private_key.clone().unwrap();
    let username = state_lock.username.clone().unwrap_or_default();

    let signer: PrivateKeySigner = private_key.parse().expect("invalid key");

    let conf = config::config();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(conf.rpc.parse().expect("invalid rpc"));

    let b = provider.get_balance(signer.address()).await.unwrap();

    Ok((signer.address().to_string(), username, b.to_string()))
}
