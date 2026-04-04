use crate::client::bridge::VirtuosoClient;
use crate::error::{Result, VirtuosoError};
use serde_json::{json, Value};

pub fn exec(code: &str, timeout: u64) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let result = client.execute_skill(code, Some(timeout))?;

    Ok(json!({
        "status": if result.ok() { "success" } else { "error" },
        "output": result.output,
        "errors": result.errors,
        "warnings": result.warnings,
        "execution_time": result.execution_time,
    }))
}

pub fn load(file: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;

    if !std::path::Path::new(file).exists() {
        return Err(VirtuosoError::NotFound(format!("file not found: {file}")));
    }

    let result = client.load_il(file)?;

    Ok(json!({
        "status": if result.ok() { "success" } else { "error" },
        "file": file,
        "output": result.output,
        "errors": result.errors,
    }))
}
