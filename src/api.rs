use axum::{extract::Path, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use crate::api::get_versions;
// use std::sync::Arc;
// use tokio::sync::RwLock;

#[derive(Serialize, Deserialize)]
struct VersionResponse {
    server_type: String,
    version: Vec<String>,
}

// Placeholder For Database
async fn get_all_versions(Path(server_type): Path<String>) -> Json<VersionResponse> {
    let available_versions: HashMap<String, Vec<String>> = [
        ("Spigot".to_string(), vec!["1.21.4", "1.20.2", "1.19.4"].iter().map(|s| s.to_string()).collect()),
        ("Paper".to_string(), vec!["1.21.4", "1.20.1", "1.19.3"].iter().map(|s| s.to_string()).collect()),
        ("Forge".to_string(), vec!["1.18.2", "1.17.1"].iter().map(|s| s.to_string()).collect()),
        ("Fabric".to_string(), vec!["1.21", "1.20"].iter().map(|s| s.to_string()).collect()),
    ]
    .iter()
    .cloned()
    .collect();

    let versions = available_versions
        .get(server_type.as_str())
        .cloned()
        .unwrap_or_else(|| vec![]);

    Json(VersionResponse {
        server_type,
        version: versions,
    })
}

pub async fn run_api() {
    let app = Router::new()
        .route("/api/version/{server_type}", get(get_all_versions));
        
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("✅ API server running on http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}
