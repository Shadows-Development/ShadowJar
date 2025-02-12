use std::{
    fs, io,
    path::Path,
    process::Command,
};
use tracing::{error, info};
use regex::Regex;

pub struct BuildConfig {
    pub server_type: String,
    pub build_tool: String,
    pub build_command: Vec<String>,
    pub output_jar: String,
}

impl BuildConfig {
    pub fn new(server_type: &str, version: &str) -> Option<Self> {
        match server_type {
            "Spigot" => Some(Self {
                server_type: server_type.to_string(),
                build_tool: "BuildTools.jar".to_string(),
                build_command: vec![
                    "-jar".to_string(),
                    "BuildTools.jar".to_string(),
                    format!("--rev {}", version),
                ],
                output_jar: format!("spigot-{}.jar", version),
            }),
            "Paper" => Some(Self {
                server_type: server_type.to_string(),
                build_tool: "Paperclip.jar".to_string(), // Placeholder, Paper fetches builds differently
                build_command: vec![],
                output_jar: format!("paper-{}.jar", version),
            }),
            "Fabric" => Some(Self {
                server_type: server_type.to_string(),
                build_tool: "fabric-installer.jar".to_string(),
                build_command: vec![
                    "java".to_string(),
                    "-jar".to_string(),
                    "fabric-installer.jar".to_string(),
                    "server".to_string(),
                    "-mcversion".to_string(),
                    version.to_string(),
                ],
                output_jar: format!("fabric-server-launch.jar"),
            }),
            _ => None, // Unsupported server type
        }
    }
}

fn extract_version(output_jar: &str) -> Option<String> {
    let re = Regex::new(r"(\d+\.\d+(\.\d+)?)").unwrap(); // Matches "1.21.4", "1.20", etc.
    re.captures(output_jar)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

pub fn run_build(config: &BuildConfig, build_path: &Path) -> io::Result<String> {
    info!(
        "🚀 Starting build for {} version {}",
        config.server_type, config.output_jar
    );

    fs::create_dir_all(&build_path)?;

    let absolute_path = dunce::canonicalize(build_path)
        .unwrap_or_else(|_| build_path.to_path_buf()) // Fallback to original path if canonicalization fails
        .to_string_lossy()
        .replace("\\", "/");

    let build_tool_path = build_path.join(&config.build_tool);
    if !build_tool_path.exists() {
        info!("📥 Copying {} into build directory...", config.build_tool);
        fs::copy(&config.build_tool, &build_tool_path)?;
    }

    // Detect OS and set the correct Bash path
    let shell = if cfg!(target_os = "windows") {
        "C:\\Program Files\\Git\\bin\\bash.exe"
    } else {
        "/bin/bash"
    };

    // Fix path formatting for Windows (Git Bash uses `/` instead of `\`)
    let build_path_str = build_path.to_string_lossy().replace("\\", "/");

    let extracted_version = extract_version(&config.output_jar)
    .unwrap_or_else(|| {
        eprintln!("⚠️ Warning: Could not extract version from '{}'", config.output_jar);
        "latest".to_string() // Default to "latest" if extraction fails
    });

    // Build the correct command string
    let command = format!(
        "cd \"{}\" && java -jar BuildTools.jar --rev {}",
        absolute_path, extracted_version // ✅ Correct: Just pass "1.21.4", not "spigot-1.21.4.jar"
    );

    info!("Executing: {}", command);

    // Run the build command in Git Bash
    let output = Command::new(shell)
        .arg("-c")
        .arg(command)
        .current_dir(build_path)
        .output()?;

    if output.status.success() {
        let jar_path = build_path.join(&config.output_jar);
        info!("✅ Build complete: {:?}", jar_path);
        Ok(jar_path.to_string_lossy().to_string())
    } else {
        error!(
            "❌ Build failed: {}\nSTDOUT: {}\nSTDERR: {}",
            config.server_type,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Err(io::Error::new(io::ErrorKind::Other, "Build failed"))
    }
}
