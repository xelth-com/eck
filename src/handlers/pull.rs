use axum::{extract::{Path, State}, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{EncryptedPacket, PullResponse};

pub async fn pull(
    State(pool): State<PgPool>,
    Path((mesh_id, instance_id)): Path<(String, String)>,
) -> Result<Json<PullResponse>, StatusCode> {
    let packets = sqlx::query_as::<_, PacketRow>(
        r#"
        DELETE FROM packets
        WHERE mesh_id = $1 
          AND target_instance_id = $2 
          AND ttl > NOW()
        RETURNING id, mesh_id, target_instance_id, sender_instance_id,
                  payload_cipher, nonce, created_at, ttl
        "#,
    )
    .bind(&mesh_id)
    .bind(&instance_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Pull failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let result: Vec<EncryptedPacket> = packets.into_iter().map(Into::into).collect();

    tracing::info!("Pull: [{}] {} got {} packets", mesh_id, instance_id, result.len());

    Ok(Json(PullResponse { 
        mesh_id,
        packets: result,
    }))
}

#[derive(sqlx::FromRow)]
struct PacketRow {
    id: Uuid,
    mesh_id: String,
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
            mesh_id: row.mesh_id,
            target_instance_id: row.target_instance_id,
            sender_instance_id: row.sender_instance_id,
            payload_cipher: row.payload_cipher,
            nonce: row.nonce,
            created_at: row.created_at,
            ttl: row.ttl,
        }
    }
}
