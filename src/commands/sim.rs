use crate::client::bridge::VirtuosoClient;
use crate::error::{Result, VirtuosoError};
use crate::ocean;
use crate::ocean::corner::CornerConfig;
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn setup(lib: &str, cell: &str, view: &str, simulator: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let skill = ocean::setup_skill(lib, cell, view, simulator);
    let result = client.execute_skill(&skill, None)?;

    if !result.ok() {
        return Err(VirtuosoError::Execution(
            result.errors.join("; "),
        ));
    }

    Ok(json!({
        "status": "success",
        "simulator": simulator,
        "design": { "lib": lib, "cell": cell, "view": view },
        "results_dir": result.output.trim().trim_matches('"'),
    }))
}

pub fn run(analysis: &str, params: &HashMap<String, String>, timeout: u64) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;

    // Check if resultsDir is set; if not, create one
    let rdir = client.execute_skill("resultsDir()", None)?;
    let rdir_val = rdir.output.trim().trim_matches('"');
    if rdir_val == "nil" || rdir_val.is_empty() {
        let default_dir = format!("/tmp/virtuoso_sim_{}", std::process::id());
        client.execute_skill(&format!("resultsDir(\"{default_dir}\")"), None)?;
        tracing::info!("auto-set resultsDir to {}", default_dir);
    }

    // Send analysis setup
    let analysis_skill = ocean::analysis_skill_simple(analysis, params);
    let analysis_result = client.execute_skill(&analysis_skill, None)?;
    if !analysis_result.ok() {
        return Err(VirtuosoError::Execution(
            analysis_result.errors.join("; "),
        ));
    }

    // Send save
    let _ = client.execute_skill("save('all)", None);

    // Execute run
    let result = client.execute_skill("run()", Some(timeout))?;
    if !result.ok() {
        return Err(VirtuosoError::Execution(
            result.errors.join("; "),
        ));
    }

    // Get actual results dir
    let rdir = client.execute_skill("resultsDir()", None)?;
    let results_dir = rdir.output.trim().trim_matches('"').to_string();

    Ok(json!({
        "status": "success",
        "analysis": analysis,
        "params": params,
        "results_dir": results_dir,
        "execution_time": result.execution_time,
    }))
}

pub fn measure(analysis: &str, exprs: &[String]) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;

    // Open results from resultsDir PSF and select result type
    let rdir = client.execute_skill("resultsDir()", None)?;
    let rdir_val = rdir.output.trim().trim_matches('"');
    if rdir_val != "nil" && !rdir_val.is_empty() {
        let open_skill = format!("openResults(\"{rdir_val}/psf\")");
        let _ = client.execute_skill(&open_skill, None);
    }
    let select_skill = format!("selectResult('{analysis})");
    let _ = client.execute_skill(&select_skill, None);

    // Execute each measure expression individually for reliability
    let mut measures = Vec::new();
    for expr in exprs {
        let result = client.execute_skill(expr, None)?;
        let value = if result.ok() {
            result.output.trim().trim_matches('"').to_string()
        } else {
            format!("ERROR: {}", result.errors.join("; "))
        };
        measures.push(json!({
            "expr": expr,
            "value": value,
        }));
    }

    Ok(json!({
        "status": "success",
        "measures": measures,
    }))
}

pub fn sweep(
    var: &str,
    from: f64,
    to: f64,
    step: f64,
    analysis: &str,
    measure_exprs: &[String],
    timeout: u64,
) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;

    // Generate value list
    let mut values = Vec::new();
    let mut v = from;
    while v <= to + step * 0.01 {
        values.push(v);
        v += step;
    }

    let skill = ocean::sweep_skill(var, &values, analysis, measure_exprs);
    let result = client.execute_skill(&skill, Some(timeout))?;

    if !result.ok() {
        return Err(VirtuosoError::Execution(
            result.errors.join("; "),
        ));
    }

    let parsed = ocean::parse_skill_list(result.output.trim());

    let mut headers = vec![var.to_string()];
    headers.extend(measure_exprs.iter().cloned());

    let rows: Vec<Value> = parsed
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, h) in headers.iter().enumerate() {
                if let Some(val) = row.get(i) {
                    obj.insert(h.clone(), json!(val));
                }
            }
            Value::Object(obj)
        })
        .collect();

    Ok(json!({
        "status": "success",
        "variable": var,
        "points": values.len(),
        "headers": headers,
        "data": rows,
        "execution_time": result.execution_time,
    }))
}

pub fn corner(file: &str, timeout: u64) -> Result<Value> {
    let content = std::fs::read_to_string(file).map_err(|e| {
        VirtuosoError::NotFound(format!("corner config not found: {file}: {e}"))
    })?;

    let config: CornerConfig = serde_json::from_str(&content).map_err(|e| {
        VirtuosoError::Config(format!("invalid corner config: {e}"))
    })?;

    let client = VirtuosoClient::from_env()?;
    let skill = ocean::corner_skill(&config);
    let result = client.execute_skill(&skill, Some(timeout))?;

    if !result.ok() {
        return Err(VirtuosoError::Execution(
            result.errors.join("; "),
        ));
    }

    let parsed = ocean::parse_skill_list(result.output.trim());

    let mut headers = vec!["corner".to_string(), "temp".to_string()];
    headers.extend(config.measures.iter().map(|m| m.name.clone()));

    let rows: Vec<Value> = parsed
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, h) in headers.iter().enumerate() {
                if let Some(val) = row.get(i) {
                    obj.insert(h.clone(), json!(val));
                }
            }
            Value::Object(obj)
        })
        .collect();

    Ok(json!({
        "status": "success",
        "corners": config.corners.len(),
        "measures": config.measures.len(),
        "headers": headers,
        "data": rows,
        "execution_time": result.execution_time,
    }))
}

pub fn results() -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let result = client.execute_skill("resultsDir()", None)?;

    if !result.ok() {
        return Err(VirtuosoError::Execution(
            result.errors.join("; "),
        ));
    }

    let dir = result.output.trim().trim_matches('"').to_string();

    // Query available result types
    let types_result = client.execute_skill(
        &format!(
            r#"let((dir files) dir="{dir}" when(isDir(dir) files=getDirFiles(dir)) files)"#
        ),
        None,
    )?;

    Ok(json!({
        "status": "success",
        "results_dir": dir,
        "contents": types_result.output.trim(),
    }))
}
