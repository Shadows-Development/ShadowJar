use shadow_jar::config::get_config;
use shadow_jar::db::{init_db, insert_version};

#[tokio::main]
async fn main() {
    println!("🚀 Initializing test database...");
    let config = get_config();
    let db = init_db(&config.paths.db_path).await;

    // Add some test data for API testing
    insert_version(db.clone(), "Spigot", "1.21.4").await;
    insert_version(db.clone(), "Spigot", "1.20.2").await;
    insert_version(db.clone(), "Paper", "1.21.4").await;

    println!("✅ Database initialized with test data.");
}
