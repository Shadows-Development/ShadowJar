use shadow_jar::db::{init_db, insert_version};
use shadow_jar::api::create_api_router;
use tokio::net::TcpListener;
use tokio::task;
use reqwest::Client;
// use std::sync::Arc;
// use tokio::sync::Mutex;

/// ✅ Utility to start the API for testing
async fn start_test_server() -> String {
    let db = init_db("test.db").await; // ✅ Use test database

    // ✅ Ensure test data exists before starting the API
    insert_version(db.clone(), "Spigot", "1.21.4").await;
    insert_version(db.clone(), "Spigot", "1.20.2").await;
    insert_version(db.clone(), "Paper", "1.21.4").await;

    let app = create_api_router(db.clone()).await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    task::spawn(async move {
        axum::serve(listener, app.into_make_service()).await.unwrap();
    });

    format!("http://{}", addr)
}

/// ✅ Test for API version retrieval
#[tokio::test]
async fn test_get_versions() {
    let server_url = start_test_server().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/versions/Spigot", server_url))
        .send()
        .await
        .unwrap();

    let status = response.status();
    let body = response.text().await.unwrap();

    // ✅ Log full response details for debugging
    eprintln!("🔹 Test Request: GET {}/api/versions/Spigot", server_url);
    eprintln!("🔹 Status Code: {:?}", status);
    eprintln!("🔹 Response Body: {}", body);

    assert_eq!(status, reqwest::StatusCode::OK, "❌ Expected 200, got {}", status);
    assert!(body.contains("Spigot"), "❌ Response does not contain 'Spigot'");
}

/// ✅ Test for 404 when requesting an unknown server type
#[tokio::test]
async fn test_unknown_server_type() {
    let server_url = start_test_server().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/versions/UnknownServer", server_url))
        .send()
        .await
        .unwrap();

    let status = response.status();
    let body = response.text().await.unwrap();

    // ✅ Log full response details for debugging
    eprintln!("🔹 Test Request: GET {}/api/versions/UnknownServer", server_url);
    eprintln!("🔹 Status Code: {:?}", status);
    eprintln!("🔹 Response Body: {}", body);

    assert_eq!(status, reqwest::StatusCode::NOT_FOUND, "❌ Expected 404, got {}", status);
}
