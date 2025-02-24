use rusqlite::{Connection, Result};
use shadow_jar::db::{get_versions, insert_version};
use std::sync::Arc;
use tokio;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_database_operations() -> Result<()> {
    let conn = Arc::new(Mutex::new(
        Connection::open_in_memory().expect("Failed to create in-memory DB"),
    ));

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

    insert_version(conn.clone(), "Spigot", "1.21.4").await;
    insert_version(conn.clone(), "Spigot", "1.20.2").await;

    let versions = get_versions(conn.clone(), "Spigot").await;

    assert!(versions.contains(&"1.21.4".to_string()));
    assert!(versions.contains(&"1.20.2".to_string()));
    assert_eq!(versions.len(), 2);

    Ok(())
}
