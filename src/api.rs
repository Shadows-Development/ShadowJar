use crate::{
    config::get_config,
    db::{get_versions as fetch_versions, DbConnection},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

#[derive(Serialize, Deserialize)]
struct VersionResponse {
    server_type: String,
    version: Vec<String>,
}

async fn get_all_versions(
    State(db): State<DbConnection>,
    Path(server_type): Path<String>,
) -> impl IntoResponse {
    let config = get_config();
    let versions = fetch_versions(db, &server_type).await;

    if versions.is_empty() {
        if config.debug.enabled {
            info!("Requested unknown server type: {}", server_type);
        }
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Server type not found", "server_type": server_type})),
        );
    }

    if config.debug.enabled {
        info!("Fetched versions for server type: {}", server_type);
    }

    (
        StatusCode::OK,
        Json(json!({ "server_type": server_type, "versions": versions })),
    )
}
async fn get_latest_version(
    State(db): State<DbConnection>,
    Path(server_type): Path<String>,
) -> impl IntoResponse {
    let db = db.lock().await;

    let query = "SELECT version FROM versions WHERE server_type = ? ORDER BY version DESC LIMIT 1";
    let latest_version: Option<String> = db.query_row(query, [&server_type], |row| row.get(0)).ok();

    match latest_version {
        Some(version) => (
            StatusCode::OK,
            Json(json!({ "server_type": server_type, "latest_version": version })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "No versions found for this server type" })),
        ),
    }
}

pub async fn create_api_router(db: DbConnection) -> Router {
    Router::new()
        .route("/api/versions/{server_type}", get(get_all_versions))
        .route(
            "/api/versions/{server_type}/latest",
            get(get_latest_version),
        )
        .with_state(db)
}
