use crate::db::{get_versions as fetch_versions, DbConnection};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct VersionResponse {
    server_type: String,
    version: Vec<String>,
}

async fn get_all_versions(
    State(db): State<DbConnection>,
    Path(server_type): Path<String>,
) -> impl IntoResponse {
    let versions = fetch_versions(db, &server_type).await;

    if versions.is_empty() {
        tracing::warn!("Requested unknown server type: {}", server_type);
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Server type not found", "server_type": server_type})),
        );
    }

    tracing::info!("Fetched versions for server type: {}", server_type);
    (
        StatusCode::OK,
        Json(json!({ "server_type": server_type, "versions": versions })),
    )
}

pub async fn create_api_router(db: DbConnection) -> Router {
    Router::new()
        .route("/api/versions/{server_type}", get(get_all_versions))
        .with_state(db)
}
