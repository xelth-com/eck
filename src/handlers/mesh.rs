use axum::{extract::{Path, State}, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::models::{MeshStatusResponse, NodeStatus};

pub async fn mesh_status(
    State(pool): State<PgPool>,
    Path(mesh_id): Path<String>,
) -> Result<Json<MeshStatusResponse>, StatusCode> {
    let nodes = sqlx::query_as::<_, NodeRow>(
        r#"
        SELECT instance_id, external_ip, port, status, last_seen
        FROM registrations 
        WHERE mesh_id = $1
        ORDER BY last_seen DESC
        "#,
    )
    .bind(&mesh_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Mesh status failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let node_statuses: Vec<NodeStatus> = nodes.into_iter().map(|n| NodeStatus {
        instance_id: n.instance_id,
        external_ip: n.external_ip,
        port: n.port as u16,
        status: n.status,
        last_seen: n.last_seen,
    }).collect();

    tracing::info!("Mesh status: [{}] {} nodes", mesh_id, node_statuses.len());

    Ok(Json(MeshStatusResponse {
        mesh_id,
        nodes: node_statuses,
    }))
}

pub async fn resolve_node(
    State(pool): State<PgPool>,
    Path((mesh_id, instance_id)): Path<(String, String)>,
) -> Result<Json<NodeStatus>, StatusCode> {
    let node = sqlx::query_as::<_, NodeRow>(
        r#"
        SELECT instance_id, external_ip, port, status, last_seen
        FROM registrations 
        WHERE mesh_id = $1 AND instance_id = $2
        "#,
    )
    .bind(&mesh_id)
    .bind(&instance_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Resolve failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match node {
        Some(n) => Ok(Json(NodeStatus {
            instance_id: n.instance_id,
            external_ip: n.external_ip,
            port: n.port as u16,
            status: n.status,
            last_seen: n.last_seen,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(sqlx::FromRow)]
struct NodeRow {
    instance_id: String,
    external_ip: String,
    port: i32,
    status: String,
    last_seen: DateTime<Utc>,
}
