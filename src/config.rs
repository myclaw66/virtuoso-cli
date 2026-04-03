use crate::error::{Result, VirtuosoError};
use dotenvy::dotenv;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub remote_host: String,
    pub remote_user: Option<String>,
    pub port: u16,
    pub jump_host: Option<String>,
    pub jump_user: Option<String>,
    pub timeout: u64,
    pub keep_remote_files: bool,
    pub spectre_cmd: String,
    pub spectre_args: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let _ = dotenv();

        let remote_host = env::var("VB_REMOTE_HOST")
            .map_err(|_| VirtuosoError::Config("VB_REMOTE_HOST not set".into()))?;

        Ok(Self {
            remote_host,
            remote_user: env::var("VB_REMOTE_USER").ok(),
            port: env::var("VB_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(65432),
            jump_host: env::var("VB_JUMP_HOST").ok(),
            jump_user: env::var("VB_JUMP_USER").ok(),
            timeout: env::var("VB_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            keep_remote_files: env::var("VB_KEEP_REMOTE_FILES")
                .ok()
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
            spectre_cmd: env::var("VB_SPECTRE_CMD")
                .ok()
                .unwrap_or_else(|| "spectre".into()),
            spectre_args: env::var("VB_SPECTRE_ARGS")
                .ok()
                .map(|v| shlex::split(&v).unwrap_or_default())
                .unwrap_or_default(),
        })
    }

    pub fn is_remote(&self) -> bool {
        !self.remote_host.is_empty()
    }

    pub fn ssh_target(&self) -> String {
        match &self.remote_user {
            Some(user) => format!("{user}@{}", self.remote_host),
            None => self.remote_host.clone(),
        }
    }

    pub fn ssh_jump(&self) -> Option<String> {
        match (&self.jump_host, &self.jump_user) {
            (Some(host), Some(user)) => Some(format!("{user}@{host}")),
            (Some(host), None) => Some(host.clone()),
            _ => None,
        }
    }
}

pub fn find_project_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join(".env").exists() {
            return Some(current);
        }
        if current.join("pyproject.toml").exists() {
            let content = std::fs::read_to_string(current.join("pyproject.toml")).ok()?;
            if content.contains("virtuoso-bridge") || content.contains("virtuoso-cli") {
                return Some(current);
            }
        }
        if !current.pop() {
            break;
        }
    }
    None
}
