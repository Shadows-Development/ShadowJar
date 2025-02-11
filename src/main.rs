use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;
use tokio::time::{sleep, Duration};
use tracing_subscriber::{fmt, prelude::*};
// use tower::ServiceBuilder;

const BUILD_TOOLS_URL: &str = "https://hub.spigotmc.org/jenkins/job/BuildTools/lastSuccessfulBuild/artifact/target/BuildTools.jar";
const BUILD_TOOLS_JAR: &str = "BuildTools.jar";
// const DB_PATH: &str = "spigot_builds.db";
const BUILD_DIR: &str = "Builds";

mod api;
mod db;
use shadow_jar::db::{insert_version, DbConnection};
// use crate::db::init_db;

static DB: OnceCell<DbConnection> = OnceCell::const_new();

async fn get_db() -> &'static DbConnection {
    DB.get_or_init(|| async {
        tracing::info!("🚀 Initializing database...");
        db::init_db().await
    })
    .await
}

// Fetches latest Minecraft version (Placeholder function)
fn get_latest_minecraft_version() -> String {
    "1.21.3".to_string() // Hardcoded for now, needs proper fetching
}

// Downloads BuildTools.jar if not present or corrupt
fn download_build_tools() -> io::Result<()> {
    if Path::new(BUILD_TOOLS_JAR).exists() {
        tracing::info!("Checking BuildTools.jar integrity...");
        let output = Command::new("java")
            .arg("-jar")
            .arg(BUILD_TOOLS_JAR)
            .arg("--help")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                tracing::info!("BuildTools.jar is valid.");
                return Ok(());
            } else {
                tracing::error!("BuildTools.jar is corrupt, redownloading...");
                fs::remove_file(BUILD_TOOLS_JAR)?;
            }
        }
    }

    tracing::info!("Downloading BuildTools...");
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(BUILD_TOOLS_URL)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36") // Fake browser request
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    if !response.status().is_success() {
        tracing::error!("Failed to download BuildTools: HTTP {}", response.status());
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to fetch BuildTools",
        ));
    }

    let mut file = fs::File::create(BUILD_TOOLS_JAR)?;
    let bytes = response
        .bytes()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    if bytes.len() < 100_000 {
        tracing::error!("Downloaded BuildTools.jar is too small, something went wrong.");
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Downloaded file is too small",
        ));
    }

    io::copy(&mut bytes.as_ref(), &mut file)?;
    tracing::info!("Download complete. Verifying integrity...");
    let verify_output = Command::new("java")
        .arg("-jar")
        .arg(BUILD_TOOLS_JAR)
        .arg("--help")
        .output();

    if let Ok(output) = verify_output {
        if output.status.success() {
            tracing::info!("BuildTools.jar verified successfully.");
            return Ok(());
        }
    }

    tracing::error!("Downloaded BuildTools.jar is corrupt.");
    fs::remove_file(BUILD_TOOLS_JAR)?;
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Downloaded file verification failed",
    ))
}

// Creates build directory structure
fn create_build_directory(server_type: &str, version: &str) -> io::Result<PathBuf> {
    let build_path = Path::new(BUILD_DIR).join(server_type).join(version);
    fs::create_dir_all(&build_path)?;
    Ok(build_path)
}

// Runs BuildTools to generate Spigot JAR in the correct directory
fn run_build_tools(server_type: &str, version: &str) -> io::Result<String> {
    let build_path = create_build_directory(server_type, version)?;
    tracing::info!(
        "Running BuildTools for version {} in {:?}...",
        version,
        build_path
    );

    // Copy BuildTools.jar into the build directory
    let build_tools_path = build_path.join("BuildTools.jar");
    if !build_tools_path.exists() {
        fs::copy(BUILD_TOOLS_JAR, &build_tools_path)?;
    }

    let output = Command::new("C:\\Program Files\\Git\\bin\\bash.exe")
        .arg("-c")
        .arg(format!(
            "cd {:?} && java -jar BuildTools.jar --rev {}",
            build_path, version
        ))
        .output()?;

    if output.status.success() {
        let jar_path = build_path.join(format!("spigot-{}.jar", version));
        tracing::info!("Build complete: {:?}", jar_path);

        // List of folders and files to delete
        let cleanup_items = vec![
            "BuildTools.jar",
            "work",
            "Spigot",
            "CraftBukkit",
            "Bukkit",
            "BuildData",
            "apache-maven-3.9.6",
        ];

        // Remove specified folders and files
        for item in &cleanup_items {
            let path = build_path.join(item);
            if path.exists() {
                if path.is_dir() {
                    fs::remove_dir_all(&path)?;
                } else {
                    fs::remove_file(&path)?;
                }
            }
        }

        tracing::info!("Build directory cleaned up, only JAR and Log file remains.");

        Ok(jar_path.to_string_lossy().to_string())
    } else {
        tracing::error!(
            "BuildTools failed with status {:?}\nSTDOUT: {}\nSTDERR: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Err(io::Error::new(io::ErrorKind::Other, "BuildTools failed"))
    }
}

// Background task for periodic build checks
async fn background_build_checker() {
    loop {
        let latest_version = get_latest_minecraft_version();
        let latest_version_clone = latest_version.clone();

        tokio::task::spawn_blocking(|| {
            download_build_tools().expect("Failed to download BuildTools")
        })
        .await
        .expect("Failed to run blocking task");

        let conn = get_db().await;

        let build_result =
            tokio::task::spawn_blocking(move || run_build_tools("Spigot", &latest_version))
                .await
                .map_err(|e| tracing::error!("Task panicked: {:?}", e))
                .ok()
                .and_then(|res| res.ok());

        if build_result.is_none() {
            eprint!("Build task was cancelled or failed")
        }

        let latest_version_clone2 = latest_version_clone.clone();
        insert_version(conn.clone(), "Spigot", &latest_version_clone2).await;

        tracing::info!("Sleeping for 6 hours before checking for new builds...");
        sleep(Duration::from_secs(6 * 3600)).await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .without_time() // ✅ Remove timestamps completely
                .with_target(false) // ✅ Remove module paths
                .with_level(true) // ✅ Keep log levels (INFO, WARN, ERROR)
                .compact(), // ✅ Compact format without extra whitespace
        )
        .init();

    // tracing::info!("🚀 Initializing database...");
    let db = get_db().await;

    tracing::info!("🚀 Starting ShadowJar API...");
    let app = api::create_api_router(db.clone()).await;

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("✅ API server running on http://localhost:8080");

    tokio::spawn(async move {
        background_build_checker().await; // ✅ Ensure it runs async
    });

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap(); // ✅ No `.await`
}
