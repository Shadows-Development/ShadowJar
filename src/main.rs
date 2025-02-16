use std::fs;
use std::io;
use std::panic;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;
use tokio::time::{sleep, Duration};
use tracing::{error, info};
use tracing_appender::rolling;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
};

const BUILD_TOOLS_URL: &str = "https://hub.spigotmc.org/jenkins/job/BuildTools/lastSuccessfulBuild/artifact/target/BuildTools.jar";
const BUILD_TOOLS_JAR: &str = "BuildTools.jar";
const BUILD_DIR: &str = "Builds";

mod api;
mod config;
mod db;
use shadow_jar::config::get_config;
use shadow_jar::db::{insert_version, DbConnection};

static DB: OnceCell<DbConnection> = OnceCell::const_new();

async fn get_db(db_name: &str) -> &'static DbConnection {
    DB.get_or_init(|| async {
        info!("🚀 Initializing database...");
        db::init_db(db_name).await
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
        info!("Checking BuildTools.jar integrity...");
        let output = Command::new("java")
            .arg("-jar")
            .arg(BUILD_TOOLS_JAR)
            .arg("--help")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                info!("BuildTools.jar is valid.");
                return Ok(());
            } else {
                error!("BuildTools.jar is corrupt, redownloading...");
                fs::remove_file(BUILD_TOOLS_JAR)?;
            }
        }
    }

    info!("Downloading BuildTools...");
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(BUILD_TOOLS_URL)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36") // Fake browser request
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    if !response.status().is_success() {
        error!("Failed to download BuildTools: HTTP {}", response.status());
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
        error!("Downloaded BuildTools.jar is too small, something went wrong.");
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Downloaded file is too small",
        ));
    }

    io::copy(&mut bytes.as_ref(), &mut file)?;
    info!("Download complete. Verifying integrity...");
    let verify_output = Command::new("java")
        .arg("-jar")
        .arg(BUILD_TOOLS_JAR)
        .arg("--help")
        .output();

    if let Ok(output) = verify_output {
        if output.status.success() {
            info!("BuildTools.jar verified successfully.");
            return Ok(());
        }
    }

    error!("Downloaded BuildTools.jar is corrupt.");
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
    info!(
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
        info!("Build complete: {:?}", jar_path);

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

        info!("Build directory cleaned up, only JAR and Log file remains.");

        Ok(jar_path.to_string_lossy().to_string())
    } else {
        error!(
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
        let config = get_config();
        let conn = get_db(&config.paths.db_path).await;

        let build_result =
            tokio::task::spawn_blocking(move || run_build_tools("Spigot", &latest_version))
                .await
                .map_err(|e| error!("Task panicked: {:?}", e))
                .ok()
                .and_then(|res| res.ok());

        if build_result.is_none() {
            eprint!("Build task was cancelled or failed")
        }

        let latest_version_clone2 = latest_version_clone.clone();
        insert_version(conn.clone(), "Spigot", &latest_version_clone2).await;

        info!("Sleeping for 6 hours before checking for new builds...");
        sleep(Duration::from_secs(6 * 3600)).await;
    }
}

fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let config = get_config();
    let log_file = rolling::daily(&config.paths.log_dir, "ShadowJar.log");
    let (file_writer, guard) = tracing_appender::non_blocking(log_file);

    let console_out = fmt::layer()
        .with_timer(fmt::time::SystemTime)
        .with_writer(std::io::stdout)
        .with_filter(EnvFilter::new("info"));
    let file_out = fmt::layer()
        .with_timer(fmt::time::SystemTime)
        .with_writer(file_writer);

    Registry::default().with(console_out).with(file_out).init();

    guard
}

fn set_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        error!("Panicked: {:?}", panic_info);
    }));
}

#[tokio::main]
async fn main() {
    let _guard = init_logging();
    set_panic_hook();

    let config = get_config();
    let db = get_db(&config.paths.db_path).await;

    info!("🚀 Starting ShadowJar API...");
    let app = api::create_api_router(db.clone()).await;

    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.api.port))
        .await
        .unwrap();
    info!(
        "✅ API server running on http://localhost:{}",
        config.api.port
    );

    tokio::spawn(async move {
        background_build_checker().await;
    });

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
