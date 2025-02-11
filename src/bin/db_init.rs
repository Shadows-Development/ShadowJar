use shadow_jar::db::{init_db, insert_version};

#[tokio::main]
async fn main() {
    println!("🚀 Initializing test database...");

    let db = init_db("shadowjar.db").await;

    // Add some test data for API testing
    insert_version(db.clone(), "Spigot", "1.21.4").await;
    insert_version(db.clone(), "Spigot", "1.20.2").await;
    insert_version(db.clone(), "Paper", "1.21.4").await;

    println!("✅ Database initialized with test data.");
}
