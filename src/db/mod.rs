use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Initialize PostgreSQL connection pool and create tables.
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;

    // Create packets table with mesh_id for routing
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS packets (
            id UUID PRIMARY KEY,
            mesh_id TEXT NOT NULL,
            target_instance_id TEXT NOT NULL,
            sender_instance_id TEXT NOT NULL,
            payload_cipher BYTEA NOT NULL,
            nonce BYTEA NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            ttl TIMESTAMPTZ NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Index for mesh-based routing (filter by mesh first)
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_packets_mesh ON packets(mesh_id)")
        .execute(&pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_packets_target ON packets(target_instance_id)",
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_packets_ttl ON packets(ttl)")
        .execute(&pool)
        .await?;

    // Create registrations table with mesh_id and status
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS registrations (
            instance_id TEXT PRIMARY KEY,
            mesh_id TEXT NOT NULL,
            external_ip TEXT NOT NULL,
            port INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'online',
            last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Index for mesh-based queries
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_registrations_mesh ON registrations(mesh_id)")
        .execute(&pool)
        .await?;

    // Index for status cleanup
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_registrations_last_seen ON registrations(last_seen)")
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
            api_key TEXT PRIMARY KEY,
            plan TEXT NOT NULL DEFAULT 'free',
            allowance INTEGER NOT NULL DEFAULT 100
        )
        "#,
    )
    .execute(&pool)
    .await?;

    tracing::info!("Database initialized (PostgreSQL) with mesh_id support");
    Ok(pool)
}

/// Delete expired packets.
pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM packets WHERE ttl < NOW()")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Mark offline nodes that haven't sent heartbeat for 20 minutes.
pub async fn mark_offline_nodes(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE registrations 
        SET status = 'offline' 
        WHERE last_seen < NOW() - INTERVAL '20 minutes'
          AND status = 'online'
        "#,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
