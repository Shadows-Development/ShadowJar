use crate::db::{get_versions as fetch_versions, DbConnection};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct VersionResponse {
    server_type: String,
    version: Vec<String>,
}

// Placeholder For Database
async fn get_all_versions(
    State(db): State<DbConnection>,
    Path(server_type): Path<String>,
) -> Json<VersionResponse> {
    let versions = fetch_versions(db, &server_type).await;

    Json(VersionResponse {
        server_type,
        version: versions,
    })
}

pub async fn create_api_router(db: DbConnection) -> Router {
    Router::new()
        .route("/api/versions/{server_type}", get(get_all_versions))
        .with_state(db)
}
