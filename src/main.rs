use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::time::{sleep, Duration};

const BUILD_TOOLS_URL: &str = "https://hub.spigotmc.org/jenkins/job/BuildTools/lastSuccessfulBuild/artifact/target/BuildTools.jar";
const BUILD_TOOLS_JAR: &str = "BuildTools.jar";
const DB_PATH: &str = "spigot_builds.db";
const BUILD_DIR: &str = "Builds";

mod api;

// Fetches latest Minecraft version (Placeholder function)
fn get_latest_minecraft_version() -> String {
    "1.21.4".to_string() // Hardcoded for now, needs proper fetching
}

// Downloads BuildTools.jar if not present or corrupt
fn download_build_tools() -> io::Result<()> {
    if Path::new(BUILD_TOOLS_JAR).exists() {
        println!("Checking BuildTools.jar integrity...");
        let output = Command::new("java")
            .arg("-jar")
            .arg(BUILD_TOOLS_JAR)
            .arg("--help")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                println!("BuildTools.jar is valid.");
                return Ok(());
            } else {
                eprintln!("BuildTools.jar is corrupt, redownloading...");
                fs::remove_file(BUILD_TOOLS_JAR)?;
            }
        }
    }

    println!("Downloading BuildTools...");
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(BUILD_TOOLS_URL)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36") // Fake browser request
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    if !response.status().is_success() {
        eprintln!("Failed to download BuildTools: HTTP {}", response.status());
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
        eprintln!("Downloaded BuildTools.jar is too small, something went wrong.");
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Downloaded file is too small",
        ));
    }

    io::copy(&mut bytes.as_ref(), &mut file)?;
    println!("Download complete. Verifying integrity...");
    let verify_output = Command::new("java")
        .arg("-jar")
        .arg(BUILD_TOOLS_JAR)
        .arg("--help")
        .output();

    if let Ok(output) = verify_output {
        if output.status.success() {
            println!("BuildTools.jar verified successfully.");
            return Ok(());
        }
    }

    eprintln!("Downloaded BuildTools.jar is corrupt.");
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
    println!(
        "Running BuildTools for version {} in {:?}...",
        version, build_path
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
        println!("Build complete: {:?}", jar_path);

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

        println!("Build directory cleaned up, only JAR and Log file remains.");

        Ok(jar_path.to_string_lossy().to_string())
    } else {
        eprintln!(
            "BuildTools failed with status {:?}\nSTDOUT: {}\nSTDERR: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Err(io::Error::new(io::ErrorKind::Other, "BuildTools failed"))
    }
}

// Stores build info in SQLite
fn store_build_info(conn: &Connection, version: &str, build_path: &str) -> Result<()> {
    let timestamp = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO spigot_builds (minecraft_version, build_timestamp, build_path) VALUES (?1, ?2, ?3)",
        params![version, timestamp, build_path],
    )?;
    Ok(())
}

// Sets up SQLite database
fn setup_database() -> Result<Connection> {
    match Connection::open(DB_PATH) {
        Ok(conn) => {
            if let Err(e) = conn.execute(
                "CREATE TABLE IF NOT EXISTS spigot_builds (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    minecraft_version TEXT NOT NULL,
                    build_timestamp TEXT NOT NULL,
                    build_path TEXT NOT NULL
                )",
                [],
            ) {
                eprintln!("Database table creation failed: {:?}", e);
                return Err(e);
            }
            Ok(conn)
        }
        Err(e) => {
            eprintln!("Failed to open SQLite database: {:?}", e);
            Err(e)
        }
    }
}

// Background task for periodic build checks
async fn background_build_checker() {
    loop {
        let latest_version = get_latest_minecraft_version();
        let latest_version_clone = latest_version.clone();

        let conn = tokio::task::spawn_blocking(setup_database)
            .await
            .map_err(|e| eprintln!("Task panicked in spawn_blocking: {:?}", e))
            .ok()
            .and_then(|res| res.ok());

        if conn.is_none() {
            eprintln!("Database setup failed, skipping build...");
            return;
        }
        let conn = conn.unwrap();

        tokio::task::spawn_blocking(|| {
            download_build_tools().expect("Failed to download BuildTools")
        })
        .await
        .expect("Failed to run blocking task");

        let build_result =
            tokio::task::spawn_blocking(move || run_build_tools("Spigot", &latest_version))
                .await
                .map_err(|e| eprintln!("Task panicked: {:?}", e))
                .ok()
                .and_then(|res| res.ok());

        if let Some(build_path) = build_result {
            let latest_version_clone2 = latest_version_clone.clone();
            tokio::task::spawn_blocking(move || {
                store_build_info(&conn, &latest_version_clone2, &build_path)
                    .expect("Failed to store build info")
            })
            .await
            .expect("Failed to store build info");
        } else {
            eprintln!("Build task was cancelled or failed");
        }

        println!("Sleeping for 6 hours before checking for new builds...");
        sleep(Duration::from_secs(6 * 3600)).await;
    }
}

#[tokio::main]
async fn main() {
    println!("🚀 Starting ShadowJar API...");
    api::run_api().await;
    // println!("Starting Spigot Build Fetcher...");
    // tokio::spawn(background_build_checker());

    // loop {
    //     tokio::time::sleep(Duration::from_secs(3600)).await;
    // }
}
