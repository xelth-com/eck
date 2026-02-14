use axum::{extract::{Path, State}, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::{EncryptedPacket, PullResponse};

pub async fn pull(
    State(pool): State<SqlitePool>,
    Path(instance_id): Path<String>,
) -> Result<Json<PullResponse>, StatusCode> {
    let rows = sqlx::query_as::<_, PacketRow>(
        r#"
        SELECT id, target_instance_id, sender_instance_id,
               payload_cipher, nonce, created_at, ttl
        FROM packets
        WHERE target_instance_id = ? AND ttl > datetime('now')
        ORDER BY created_at ASC
        "#,
    )
    .bind(&instance_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Pull failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Delete delivered packets
    if !rows.is_empty() {
        let ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();
        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let query = format!(
            "DELETE FROM packets WHERE id IN ({})",
            placeholders.join(",")
        );
        let mut q = sqlx::query(&query);
        for id in &ids {
            q = q.bind(id);
        }
        let _ = q.execute(&pool).await;
    }

    let packets: Vec<EncryptedPacket> = rows
        .into_iter()
        .filter_map(|r| r.try_into().ok())
        .collect();

    tracing::info!("Pull by {}: {} packets", instance_id, packets.len());

    Ok(Json(PullResponse { packets }))
}

#[derive(sqlx::FromRow)]
struct PacketRow {
    id: String,
    target_instance_id: String,
    sender_instance_id: String,
    payload_cipher: Vec<u8>,
    nonce: Vec<u8>,
    created_at: String,
    ttl: String,
}

impl TryFrom<PacketRow> for EncryptedPacket {
    type Error = chrono::ParseError;

    fn try_from(row: PacketRow) -> Result<Self, Self::Error> {
        Ok(EncryptedPacket {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            target_instance_id: row.target_instance_id,
            sender_instance_id: row.sender_instance_id,
            payload_cipher: row.payload_cipher,
            nonce: row.nonce,
            created_at: row.created_at.parse::<DateTime<Utc>>()?,
            ttl: row.ttl.parse::<DateTime<Utc>>()?,
        })
    }
}
