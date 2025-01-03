use aes_gcm::Aes256Gcm;
use tokio::sync::Mutex;

mod config;
mod db;
mod errors;
mod eth;
mod ui;

use crate::ui::create_account::create_account;
use crate::ui::dashboard::add_friend;
use crate::ui::dashboard::check_and_update_ssk;
use crate::ui::dashboard::select_friends;
use crate::ui::dashboard::state_is_filled;
use crate::ui::dashboard::{read_messages, send_message};
use crate::ui::login::check_if_password_looks_fine;
use crate::ui::login::login;
use crate::ui::notifications::get_incoming_friend_requests;
use crate::ui::notifications::get_pending_outgoing_requests;
use crate::ui::profiles::get_account_0x;

#[derive(Default)]
pub struct LocalEncryptionCipher {
    pub username: Option<String>,
    pub cipher: Option<Aes256Gcm>,
    pub private_key: Option<String>,
}

unsafe impl Send for LocalEncryptionCipher {}
unsafe impl Sync for LocalEncryptionCipher {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // Register a `State` to be managed by Tauri
        // We need write access to it so we wrap it in a `Mutex`
        .manage(Mutex::new(LocalEncryptionCipher::default()))
        // Add a command we can use to check
        .invoke_handler(tauri::generate_handler![
            create_account,
            login,
            state_is_filled,
            check_if_password_looks_fine,
            add_friend,
            select_friends,
            check_and_update_ssk,
            read_messages,
            send_message,
            get_incoming_friend_requests,
            get_pending_outgoing_requests,
            get_account_0x,
        ])
        // Use the setup hook to execute setup related tasks
        // Runs before the main loop, so no windows are yet created
        //.setup(|app| {
        // Spawn setup as a non-blocking task so the windows can be
        // created and ran while it executes
        //spawn(load_config(app.handle().clone()));
        // The hook expects an Ok result
        //    Ok(())
        //})
        // Run the app
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
