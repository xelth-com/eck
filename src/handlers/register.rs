use axum::{extract::State, http::StatusCode, Json};
use sqlx::SqlitePool;

use crate::models::{RegisterRequest, RegisterResponse};

pub async fn register(
    State(pool): State<SqlitePool>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    sqlx::query(
        r#"
        INSERT INTO registrations (instance_id, external_ip, port, last_seen)
        VALUES (?, ?, ?, datetime('now'))
        ON CONFLICT(instance_id) DO UPDATE SET
            external_ip = excluded.external_ip,
            port = excluded.port,
            last_seen = datetime('now')
        "#,
    )
    .bind(&req.instance_id)
    .bind(&req.external_ip)
    .bind(req.port as i64)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Register failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Heartbeat from {} ({}:{})", req.instance_id, req.external_ip, req.port);

    Ok(Json(RegisterResponse {
        ok: true,
        instance_id: req.instance_id,
    }))
}
