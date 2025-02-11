use shadow_jar::db::init_db;
use shadow_jar::api::create_api_router;
use tokio::net::TcpListener;
use tokio::task;
use reqwest::Client;

/// ✅ Utility to start the API for testing
async fn start_test_server() -> String {
    let db = init_db("test.db").await; // ✅ Returns Arc<Mutex<Connection>>
    let app = create_api_router(db.clone()).await; // ✅ Pass the correct type

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
        .get(format!("{}/api/version/Spigot", server_url))
        .send()
        .await
        .unwrap();
    let status = response.status();
    let test = response.text().await.unwrap();
    println!("{}", test);
    assert_eq!(status, reqwest::StatusCode::OK);

    // let body = response.text().await.unwrap();
    println!("Response: {}", test);
    
    assert!(test.contains("Spigot"));
}

/// ✅ Test for 404 when requesting an unknown server type
#[tokio::test]
async fn test_unknown_server_type() {
    let server_url = start_test_server().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/version/UnknownServer", server_url))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}
