mod db;
mod handlers;
mod models;

use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use tokio::time::{interval, Duration};
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "eck=info".into()))
        .init();

    // Database
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:eck.db?mode=rwc".into());
    let pool = db::init_pool(&db_url)
        .await
        .expect("Failed to initialize database");

    // Background cleanup task
    let cleanup_pool = pool.clone();
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(60));
        loop {
            tick.tick().await;
            match db::cleanup_expired(&cleanup_pool).await {
                Ok(n) if n > 0 => tracing::info!("Cleaned up {n} expired packets"),
                Err(e) => tracing::warn!("Cleanup error: {e}"),
                _ => {}
            }
        }
    });

    // Routes — all under /E/ prefix
    let app = Router::new()
        .route("/E/health", get(handlers::health))
        .route("/E/register", post(handlers::register))
        .route("/E/push", post(handlers::push))
        .route("/E/pull/{id}", get(handlers::pull))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    // Bind
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3200);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("Eck relay listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
