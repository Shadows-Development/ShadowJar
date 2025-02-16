use config::{Config, File};
use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Settings {
    pub build: BuildConfig,
    pub paths: PathConfig,
    pub api: ApiConfig,
    pub debug: DebugConfig,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct BuildConfig {
    pub max_parallel_builds: u32,
    pub default_server_type: String,
    pub enable_cleanup: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PathConfig {
    pub build_dir: String,
    pub log_dir: String,
    pub db_path: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiConfig {
    pub port: u16,
    // pub allowed_origins: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DebugConfig {
    pub enabled: bool,
}

#[allow(dead_code)]
pub static CONFIG: Lazy<Settings> = Lazy::new(|| {
    Config::builder()
        .add_source(File::with_name("config"))
        .build()
        .expect("Failed to read config file")
        .try_deserialize()
        .expect("Failed to parse config")
});

#[allow(dead_code)]
pub fn get_config() -> &'static Settings {
    &CONFIG
}
