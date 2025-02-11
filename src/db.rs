use rusqlite::{params, Connection, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type DbConnection = Arc<Mutex<Connection>>;

/// Initializes the database and creates the `versions` table if it doesn't exist.
pub async fn init_db() -> DbConnection {
    let conn = Connection::open("shadowjar.db").expect("Failed to open database");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS versions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            server_type TEXT NOT NULL,
            version TEXT NOT NULL
        )",
        [],
    )
    .expect("Failed to create table");

    Arc::new(Mutex::new(conn))
}

/// Inserts a new Server version into the database.
pub async fn insert_version(db: DbConnection, server_type: &str, version: &str) {
    let conn = db.lock().await;
    conn.execute(
        "INSERT INTO versions (server_type, version) VALUES (?1,?2)",
        params![server_type, version],
    )
    .expect("Failed to insert version");
}

pub async fn get_versions(db: DbConnection, server_type: &str) -> Vec<String> {
    let conn = db.lock().await;
    let mut stmt = conn
        .prepare("SELECT version FROM versions WHERE server_type = ?1")
        .expect("Failed to prepare query");

    let rows = stmt
        .query_map(params![server_type], |row| row.get(0))
        .expect("Failed to fetch versions");

    rows.filter_map(Result::ok).collect()
}
