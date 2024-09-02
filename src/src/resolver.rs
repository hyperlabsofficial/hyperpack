use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use regex::Regex;
use log::{info, error, debug};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub struct PluginManager;

impl PluginManager {
    pub fn resolve(&self, path: &str) -> Option<String> {
        // Placeholder for actual plugin resolution logic
        None
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    paths: HashMap<String, String>,
    extensions: Vec<String>,
}

fn load_config(config_path: &str) -> Result<Config, String> {
    let config_file = fs::read_to_string(config_path)
        .map_err(|err| format!("Failed to read config file: {}", err))?;
    serde_json::from_str(&config_file).map_err(|err| format!("Failed to parse config file: {}", err))
}

pub fn resolve_path(base: &str, import_path: &str, plugins: &PluginManager, config_path: &str) -> String {
    let re = Regex::new(r"(?P<path>[./\w-]+)(?:#(?P<fragment>[\w-]+))?").unwrap();
    let config = match load_config(config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("{}", err);
            return "".to_string();
        }
    };

    let mut resolved_path: PathBuf;
    let base_path = Path::new(base);
    let base_dir = base_path.parent().unwrap_or_else(|| Path::new(""));

    debug!("Base directory: {:?}", base_dir);
    debug!("Import path: {}", import_path);

    if let Some(new_path) = plugins.resolve(import_path) {
        info!("Resolved path via plugins: {}", new_path);
        return new_path;
    }

    if let Some(caps) = re.captures(import_path) {
        let path_match = caps.name("path").map_or("", |m| m.as_str());
        let fragment_match = caps.name("fragment").map_or("", |m| m.as_str());

        resolved_path = if path_match.starts_with("./") || path_match.starts_with("../") {
            base_dir.join(path_match)
        } else {
            PathBuf::from(path_match)
        };

        if !fragment_match.is_empty() {
            let mut new_path = resolved_path.clone();
            new_path.set_extension(format!("{}.ext", fragment_match));
            resolved_path = new_path;
        }

        if resolved_path.to_str().unwrap_or("").contains("..") {
            let normalized_path = resolved_path.canonicalize().unwrap_or_else(|_| resolved_path.clone());
            resolved_path = normalized_path;
        }

        if !resolved_path.exists() {
            if let Some(alternative_path) = try_alternate_resolutions(&resolved_path, &config) {
                resolved_path = alternative_path;
            }
        }
    } else {
        resolved_path = base_dir.join(import_path);
    }

    if let Some(env_path) = load_env_variable_path("RESOLVED_PATH") {
        info!("Path resolved via environment variable: {:?}", env_path);
        resolved_path = env_path;
    }

    if !verify_path_security(&resolved_path) {
        error!("Path {} is not secure or does not exist", resolved_path.display());
        return "".to_string();
    }

    resolved_path.to_str().unwrap_or("").to_string()
}

fn try_alternate_resolutions(path: &PathBuf, config: &Config) -> Option<PathBuf> {
    let extensions = config.extensions.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    for ext in &extensions {
        let mut new_path = path.clone();
        new_path.set_extension(ext);
        if new_path.exists() {
            info!("Resolved path with alternate extension: {:?}", new_path);
            return Some(new_path);
        }
    }
    None
}

fn load_env_variable_path(variable_name: &str) -> Option<PathBuf> {
    env::var(variable_name).ok().map(PathBuf::from)
}

fn verify_path_security(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.is_file() && !metadata.permissions().readonly()
    } else {
        false
    }
}

fn main() {
    env_logger::init();
    let plugins = PluginManager;
    let base = "../main.rs";
    let import_path = "./utils#fragment";
    let config_path = "config.json";
    
    let resolved_path = resolve_path(base, import_path, &plugins, config_path);
    println!("Resolved Path: {}", resolved_path);
}