use std::path::PathBuf;

use rusqlite::Connection;

pub fn get_persistent_connection(path: &PathBuf) -> Connection {
    // TODO: Create parent dir if not exists
    Connection::open(path).expect("DB_PCONN_FAILED")
}

pub fn get_in_memory_connection() -> Connection {
    Connection::open_in_memory().expect("DB_IMCONN_FAILED")
}
