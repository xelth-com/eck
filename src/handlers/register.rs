use axum::{extract::State, http::StatusCode, Json};
use sqlx::PgPool;

use crate::models::{RegisterRequest, RegisterResponse};

pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let status = req.status.unwrap_or_else(|| "online".to_string());
    
    sqlx::query(
        r#"
        INSERT INTO registrations (instance_id, mesh_id, external_ip, port, status, last_seen)
        VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT(instance_id) DO UPDATE SET
            mesh_id = EXCLUDED.mesh_id,
            external_ip = EXCLUDED.external_ip,
            port = EXCLUDED.port,
            status = EXCLUDED.status,
            last_seen = NOW()
        "#,
    )
    .bind(&req.instance_id)
    .bind(&req.mesh_id)
    .bind(&req.external_ip)
    .bind(req.port as i32)
    .bind(&status)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Register failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(
        "Heartbeat: {} ({}) at {}:{} [{}]",
        req.instance_id, req.mesh_id, req.external_ip, req.port, status
    );

    Ok(Json(RegisterResponse {
        ok: true,
        instance_id: req.instance_id,
        mesh_id: req.mesh_id,
        status,
    }))
}
