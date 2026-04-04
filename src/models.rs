use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Success,
    Failure,
    Partial,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtuosoResult {
    pub status: ExecutionStatus,
    pub output: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub execution_time: Option<f64>,
    pub metadata: HashMap<String, String>,
}

impl VirtuosoResult {
    pub fn ok(&self) -> bool {
        self.status == ExecutionStatus::Success
    }

    pub fn success(output: impl Into<String>) -> Self {
        Self {
            status: ExecutionStatus::Success,
            output: output.into(),
            errors: Vec::new(),
            warnings: Vec::new(),
            execution_time: None,
            metadata: HashMap::new(),
        }
    }

    pub fn error(errors: Vec<String>) -> Self {
        Self {
            status: ExecutionStatus::Error,
            output: String::new(),
            errors,
            warnings: Vec::new(),
            execution_time: None,
            metadata: HashMap::new(),
        }
    }

    pub fn save_json(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        std::fs::write(path, json)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub status: ExecutionStatus,
    pub tool_version: Option<String>,
    pub data: HashMap<String, Vec<f64>>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl SimulationResult {
    pub fn ok(&self) -> bool {
        self.status == ExecutionStatus::Success
    }

    pub fn save_json(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        std::fs::write(path, json)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTaskResult {
    pub success: bool,
    pub returncode: i32,
    pub stdout: String,
    pub stderr: String,
    pub remote_dir: Option<String>,
    pub error: Option<String>,
    pub timings: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSshEnv {
    pub remote_host: String,
    pub remote_user: Option<String>,
    pub jump_host: Option<String>,
    pub jump_user: Option<String>,
}

fn default_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelState {
    #[serde(default = "default_version")]
    pub version: u32,
    pub port: u16,
    pub pid: u32,
    pub remote_host: String,
    pub setup_path: Option<String>,
}

impl TunnelState {
    pub fn save(&self) -> std::io::Result<()> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("virtuoso_bridge");
        std::fs::create_dir_all(&cache_dir)?;
        let state_path = cache_dir.join("state.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        std::fs::write(state_path, json)
    }

    pub fn load() -> std::io::Result<Option<Self>> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("virtuoso_bridge");
        let state_path = cache_dir.join("state.json");
        if !state_path.exists() {
            return Ok(None);
        }
        let json = std::fs::read_to_string(state_path)?;
        serde_json::from_str(&json)
            .map(Some)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }

    pub fn clear() -> std::io::Result<()> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("virtuoso_bridge");
        let state_path = cache_dir.join("state.json");
        if state_path.exists() {
            std::fs::remove_file(state_path)?;
        }
        Ok(())
    }
}
