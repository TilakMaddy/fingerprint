use rusqlite::Connection;

pub fn create_tables_if_not_exists(conn: &Connection) {
    conn.execute_batch(
        r#"

    BEGIN;

    CREATE TABLE IF NOT EXISTS
      `Personal` (
        `id` integer not null primary key autoincrement,
        `created_at` datetime not null default CURRENT_TIMESTAMP,
        `username` varchar(255) null,
        `password_hash` VARCHAR(255) null,
        `password_salt` BLOB null,
        `private_key_nonce` BLOB null,
        `enc_private_key` BLOB null,
        unique (`id`)
    );

    CREATE TABLE IF NOT EXISTS
      `Friend` (
        `id` integer not null primary key autoincrement,
        `created_at` datetime not null default CURRENT_TIMESTAMP,
        `name` varchar(255) null,
        `username` varchar(255) null,
        `public_address` varchar(255) null,
        unique (`id`)
      );


    CREATE TABLE IF NOT EXISTS
      `Keys` (
        `id` integer not null primary key autoincrement,
        `created_at` datetime not null default CURRENT_TIMESTAMP,
        `friend_id` INTEGER null,
        `my_secret_nonce` BLOB null,
        `enc_my_secret` BLOB null,
        `shared_secret_nonce` BLOB null,
        `username` varchar(255) null,
        `enc_shared_secret` BLOB null,
        unique (`id`)
      );


    COMMIT;

    "#,
    )
    .expect("SQL EXEC failed");
}
