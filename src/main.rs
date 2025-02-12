use std::panic;
use std::path::Path;
use build::{BuildConfig, run_build};
use tokio::net::TcpListener;
use tokio::sync::OnceCell;
use tracing::{error, info};
use tracing_appender::rolling;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
};

mod api;
mod db;
mod build;
use shadow_jar::db::DbConnection;
// use shadow_jar::build::{BuildConfig, run_build};

static DB: OnceCell<DbConnection> = OnceCell::const_new();

async fn get_db() -> &'static DbConnection {
    DB.get_or_init(|| async {
        info!("🚀 Initializing database...");
        db::init_db("shadowjar.db").await
    })
    .await
}

fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_file = rolling::daily("logs", "ShadowJar.log");
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
    let db = get_db().await;

    info!("🚀 Starting ShadowJar API...");
    let app = api::create_api_router(db.clone()).await;

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("✅ API server running on http://localhost:8080");

    let server_type = "Spigot";
    let version = "1.21.4";

    if let Some(config) = BuildConfig::new(server_type, version) {
        let build_path = Path::new("Builds").join(&config.server_type).join(version);

        match run_build(&config, &build_path) {
            Ok(path) => info!("✅ Successfully built {}", path),
            Err(e) => error!("❌ Build failed: {}", e),
        }
    } else {
        error!("❌ Unsupported server type: {}", server_type);
    }

    // tokio::spawn(async move {
    //     background_build_checker().await;
    // });

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
