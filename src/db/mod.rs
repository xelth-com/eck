use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;

/// Initialize SQLite database with WAL mode.
pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    // Enable WAL mode for concurrent reads
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;

    // Create tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS packets (
            id TEXT PRIMARY KEY,
            target_instance_id TEXT NOT NULL,
            sender_instance_id TEXT NOT NULL,
            payload_cipher BLOB NOT NULL,
            nonce BLOB NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            ttl TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_packets_target
            ON packets(target_instance_id);

        CREATE INDEX IF NOT EXISTS idx_packets_ttl
            ON packets(ttl);
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS registrations (
            instance_id TEXT PRIMARY KEY,
            external_ip TEXT NOT NULL,
            port INTEGER NOT NULL,
            last_seen TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
            api_key TEXT PRIMARY KEY,
            plan TEXT NOT NULL DEFAULT 'free',
            allowance INTEGER NOT NULL DEFAULT 100
        );
        "#,
    )
    .execute(&pool)
    .await?;

    tracing::info!("Database initialized (SQLite WAL mode)");
    Ok(pool)
}

/// Delete expired packets.
pub async fn cleanup_expired(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM packets WHERE ttl < datetime('now')")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
