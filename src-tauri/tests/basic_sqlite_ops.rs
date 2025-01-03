#[cfg(test)]
mod basic_sqlite_tests {
    use rusqlite::Connection;

    #[test]
    fn test_sqlite() {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS person (
            id   INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            data BLOB
        )",
            (),
        )
        .unwrap();
    }
}
