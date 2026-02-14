use axum::{extract::{Path, State}, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{EncryptedPacket, PullResponse};

pub async fn pull(
    State(pool): State<PgPool>,
    Path(instance_id): Path<String>,
) -> Result<Json<PullResponse>, StatusCode> {
    // Fetch and delete in one query (RETURNING)
    let packets = sqlx::query_as::<_, PacketRow>(
        r#"
        DELETE FROM packets
        WHERE target_instance_id = $1 AND ttl > NOW()
        RETURNING id, target_instance_id, sender_instance_id,
                  payload_cipher, nonce, created_at, ttl
        "#,
    )
    .bind(&instance_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Pull failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let result: Vec<EncryptedPacket> = packets.into_iter().map(Into::into).collect();

    tracing::info!("Pull by {}: {} packets", instance_id, result.len());

    Ok(Json(PullResponse { packets: result }))
}

#[derive(sqlx::FromRow)]
struct PacketRow {
    id: Uuid,
    target_instance_id: String,
    sender_instance_id: String,
    payload_cipher: Vec<u8>,
    nonce: Vec<u8>,
    created_at: DateTime<Utc>,
    ttl: DateTime<Utc>,
}

impl From<PacketRow> for EncryptedPacket {
    fn from(row: PacketRow) -> Self {
        EncryptedPacket {
            id: row.id,
            target_instance_id: row.target_instance_id,
            sender_instance_id: row.sender_instance_id,
            payload_cipher: row.payload_cipher,
            nonce: row.nonce,
            created_at: row.created_at,
            ttl: row.ttl,
        }
    }
}
