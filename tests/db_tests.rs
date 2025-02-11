use rusqlite::{Connection, Result};
use std::sync::Arc;
use tokio;
use tokio::sync::Mutex;
use shadow_jar::db::{get_versions, insert_version};

#[tokio::test]
async fn test_database_operations() -> Result<()> {
    // ✅ Wrap the connection in `Arc<Mutex<Connection>>` for async compatibility
    let conn = Arc::new(Mutex::new(
        Connection::open_in_memory().expect("Failed to create in-memory DB"),
    ));

    // ✅ Create the versions table inside the locked connection
    {
        let conn = conn.lock().await;
        conn.execute(
            "CREATE TABLE versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_type TEXT NOT NULL,
                version TEXT NOT NULL
            )",
            [],
        )
        .expect("Failed to create test table");
    }

    // ✅ Insert test data
    insert_version(conn.clone(), "Spigot", "1.21.4").await;
    insert_version(conn.clone(), "Spigot", "1.20.2").await;

    // ✅ Fetch inserted versions
    let versions = get_versions(conn.clone(), "Spigot").await;

    // ✅ Assertions
    assert!(versions.contains(&"1.21.4".to_string()));
    assert!(versions.contains(&"1.20.2".to_string()));
    assert_eq!(versions.len(), 2);

    Ok(())
}
