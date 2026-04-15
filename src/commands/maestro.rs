use crate::client::bridge::VirtuosoClient;
use crate::commands::schematic::parse_skill_json;
use crate::error::{Result, VirtuosoError};
use serde_json::{json, Value};

pub fn open(lib: &str, cell: &str, view: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let skill = client.maestro.open_session(lib, cell, view);
    let r = client.execute_skill(&skill, None)?;
    if !r.skill_ok() {
        return Err(VirtuosoError::Execution(format!(
            "Failed to open maestro session: {}",
            r.output
        )));
    }
    Ok(json!({
        "status": "success",
        "session": r.output.trim_matches('"'),
        "lib": lib, "cell": cell, "view": view,
    }))
}

pub fn close(session: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let skill = client.maestro.close_session(session);
    let r = client.execute_skill(&skill, None)?;
    Ok(json!({
        "status": if r.skill_ok() { "success" } else { "error" },
        "session": session,
        "output": r.output,
    }))
}

pub fn list_sessions() -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let skill = client.maestro.list_sessions();
    let r = client.execute_skill(&skill, None)?;
    if !r.skill_ok() {
        return Err(VirtuosoError::Execution(format!(
            "Failed to list sessions: {}",
            r.output
        )));
    }
    Ok(parse_skill_json(&r.output))
}

pub fn set_var(session: &str, name: &str, value: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let skill = client.maestro.set_var(session, name, value);
    let r = client.execute_skill(&skill, None)?;
    Ok(json!({
        "status": if r.skill_ok() { "success" } else { "error" },
        "session": session, "variable": name, "value": value,
        "output": r.output,
    }))
}

pub fn get_analyses(session: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let skill = client.maestro.get_analyses(session);
    let r = client.execute_skill(&skill, None)?;
    Ok(json!({
        "session": session,
        "analyses": r.output.trim_matches('"'),
    }))
}

pub fn set_analysis(session: &str, analysis_type: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    // Resolve setup name from session, then enable analysis type.
    let setup_skill = format!(r#"car(maeGetSetup(?session "{session}"))"#);
    let setup_r = client.execute_skill(&setup_skill, None)?;
    if !setup_r.skill_ok() {
        return Err(VirtuosoError::NotFound(format!(
            "no setup found for session '{session}'"
        )));
    }
    let setup = setup_r.output.trim_matches('"');
    let skill = client.maestro.set_analysis(setup, analysis_type);
    let r = client.execute_skill(&skill, None)?;
    Ok(json!({
        "status": if r.skill_ok() { "success" } else { "error" },
        "session": session,
        "setup": setup,
        "analysis": analysis_type,
    }))
}

pub fn run(session: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let skill = client.maestro.run_simulation(session);
    let r = client.execute_skill(&skill, None)?;
    Ok(json!({
        "status": if r.skill_ok() { "launched" } else { "error" },
        "session": session,
        "output": r.output.trim_matches('"'),
    }))
}

pub fn add_output(session: &str, name: &str, expr: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let skill = client.maestro.add_output(session, name, expr);
    let r = client.execute_skill(&skill, None)?;
    Ok(json!({
        "status": if r.skill_ok() { "success" } else { "error" },
        "session": session, "name": name, "expression": expr,
        "output": r.output,
    }))
}

pub fn save(session: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let skill = client.maestro.save_setup(session);
    let r = client.execute_skill(&skill, None)?;
    Ok(json!({
        "status": if r.skill_ok() { "success" } else { "error" },
        "session": session,
    }))
}

pub fn export(session: &str, path: &str) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let skill = client.maestro.export_results(session, path);
    let r = client.execute_skill(&skill, None)?;
    Ok(json!({
        "status": if r.skill_ok() { "success" } else { "error" },
        "session": session,
        "path": path,
        "output": r.output,
    }))
}
