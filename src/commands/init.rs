use crate::error::{Result, VirtuosoError};
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;

const ENV_TEMPLATE: &str = r#"# Virtuoso CLI Configuration
# Remote host (SSH alias or hostname)
VB_REMOTE_HOST=

# Remote user (optional, defaults to current user)
# VB_REMOTE_USER=

# SSH port (default: 65432)
VB_PORT=65432

# Jump/bastion host (optional)
# VB_JUMP_HOST=
# VB_JUMP_USER=

# Timeout in seconds (default: 30)
VB_TIMEOUT=30

# Keep remote files after stopping (default: false)
VB_KEEP_REMOTE_FILES=false

# Spectre command (default: spectre)
# VB_SPECTRE_CMD=spectre

# Spectre extra arguments
# VB_SPECTRE_ARGS=
"#;

pub fn run(if_not_exists: bool) -> Result<Value> {
    let env_path = Path::new(".env");

    if env_path.exists() {
        if if_not_exists {
            return Ok(json!({
                "status": "skipped",
                "reason": ".env already exists",
                "path": ".env",
            }));
        }
        return Err(VirtuosoError::Conflict(
            ".env already exists (use --if-not-exists to skip)".into(),
        ));
    }

    let mut file = std::fs::File::create(env_path)?;
    file.write_all(ENV_TEMPLATE.as_bytes())?;

    Ok(json!({
        "status": "created",
        "path": ".env",
        "next_step": "Edit .env and set VB_REMOTE_HOST, then run: virtuoso tunnel start",
    }))
}
