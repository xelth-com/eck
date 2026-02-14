use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::{PushRequest, PushResponse};

/// Maximum payload size: 1 MB.
const MAX_PAYLOAD_SIZE: usize = 1_048_576;

pub async fn push(
    State(pool): State<SqlitePool>,
    Json(req): Json<PushRequest>,
) -> Result<Json<PushResponse>, StatusCode> {
    if req.payload_cipher.len() > MAX_PAYLOAD_SIZE {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    let id = Uuid::new_v4();
    let ttl_seconds = req.ttl_seconds.unwrap_or(3600); // default 1 hour
    let ttl = Utc::now() + Duration::seconds(ttl_seconds as i64);

    sqlx::query(
        r#"
        INSERT INTO packets (id, target_instance_id, sender_instance_id, payload_cipher, nonce, ttl)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id.to_string())
    .bind(&req.target_instance_id)
    .bind(&req.sender_instance_id)
    .bind(&req.payload_cipher)
    .bind(&req.nonce)
    .bind(ttl.format("%Y-%m-%d %H:%M:%S").to_string())
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Push failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(
        "Packet {} stored: {} -> {} (ttl {}s)",
        id, req.sender_instance_id, req.target_instance_id, ttl_seconds
    );

    Ok(Json(PushResponse { ok: true, packet_id: id }))
}
