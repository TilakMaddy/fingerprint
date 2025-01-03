use rusqlite::Connection;

use crate::{db, errors::Result};
use std::{
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};

static DB_FILE_NAME: &str = "db.version_0_0_1.sqlite";
static RPC_URL: &str = "http://localhost:8545";
static CONTRACT_ADDRESS: &str = "0x5FbDB2315678afecb367f032d93F642f64180aa3";

static DB_LOC: LazyLock<DbLoc> = LazyLock::new(|| {
    // TODO: Switch the condition
    if !cfg!(debug_assertions) {
        DbLoc::InMemory
    } else {
        let platform =
            platform_dirs::AppDirs::new(Some("TheTunnel"), true).expect("DB_FAIL_TO_INIT");
        DbLoc::Persistent(platform.data_dir.join(DB_FILE_NAME))
    }
});

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        Config::load().unwrap_or_else(|ex| {
            panic!("FATAL - Loading config failed - Cause {:?}", ex);
        })
    })
}

pub enum DbLoc {
    InMemory,
    Persistent(PathBuf),
}

pub struct Config {
    pub conn: &'static mut Connection,
    pub rpc: &'static str,
    pub contract_address: &'static str,
}

unsafe impl Send for Config {}
unsafe impl Sync for Config {}

impl Config {
    pub fn load() -> Result<Self> {
        // Chose the connetion type based on the Database location
        let conn = Box::new(match &*DB_LOC {
            DbLoc::InMemory => db::conn::get_in_memory_connection(),
            DbLoc::Persistent(db_file) => db::conn::get_persistent_connection(db_file),
        });

        // Create tables if not exists
        db::setup::create_tables_if_not_exists(&conn);
        let config = Config {
            conn: Box::leak(conn),
            contract_address: CONTRACT_ADDRESS,
            rpc: RPC_URL,
        };

        Ok(config)
    }
}

#[cfg(test)]
mod config_tests {
    use super::config;

    #[test]
    fn database_connection() {
        let config = config();
        let test_query: usize = config
            .conn
            .query_row("select 1;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(test_query, 1);
    }
}
