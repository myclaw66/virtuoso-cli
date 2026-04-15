use crate::client::bridge::VirtuosoClient;
use crate::error::{Result, VirtuosoError};
use serde_json::{json, Value};

/// List all open Virtuoso windows with their names.
///
/// Window names reveal the current mode, e.g.:
///   "ADE Explorer Editing: LIB/CELL/maestro"
///   "ADE Explorer Reading: ..."
///   "Virtuoso Schematic Editor"
pub fn list() -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let r = client.execute_skill(&client.window.list_windows(), None)?;
    if !r.ok() {
        return Err(VirtuosoError::Execution(format!(
            "failed to list windows: {}",
            r.errors.join("; ")
        )));
    }
    let windows = parse_window_json(&r.output);
    // Annotate each window with a derived mode field
    let windows = annotate_modes(windows);
    Ok(json!({ "windows": windows }))
}

/// Dismiss the currently active blocking dialog.
///
/// With --dry-run, reports the dialog name without clicking anything.
/// action "cancel" (default): clicks Cancel / closes dialog.
/// action "ok": attempts hiSendOK — may not be supported by all dialog types.
pub fn dismiss_dialog(action: &str, dry_run: bool) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    if dry_run {
        let r = client.execute_skill(&client.window.get_dialog_info(), None)?;
        let raw = r.output.trim_matches('"');
        let active = r.skill_ok() && raw != "no-dialog";
        return Ok(json!({
            "dialog": if active { raw } else { "none" },
            "active": active,
            "dry_run": true,
        }));
    }
    let r = client.execute_skill(&client.window.dismiss_dialog(action), None)?;
    let dismissed = r.skill_ok() && r.output.trim_matches('"') != "no-dialog";
    Ok(json!({
        "status": if dismissed { "dismissed" } else { "no-dialog" },
        "action": action,
    }))
}

/// Capture a screenshot of the current (or pattern-matched) Virtuoso window.
///
/// Saves to --path as PNG. Requires IC23.1+ (hiGetWindowScreenDump).
pub fn screenshot(path: &str, window_pattern: Option<&str>) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;
    let skill = match window_pattern {
        Some(pat) => client.window.screenshot_by_pattern(path, pat),
        None => client.window.screenshot(path),
    };
    let r = client.execute_skill(&skill, None)?;
    if !r.skill_ok() {
        let detail = if r.output.is_empty() {
            r.errors.join("; ")
        } else {
            r.output.clone()
        };
        return Err(VirtuosoError::Execution(format!(
            "screenshot failed: {}",
            detail
        )));
    }
    if r.output.trim_matches('"') == "no-match" {
        return Err(VirtuosoError::NotFound(format!(
            "no window matching pattern '{}'",
            window_pattern.unwrap_or("")
        )));
    }
    Ok(json!({
        "status": "saved",
        "path": path,
    }))
}

/// Derive a mode string from a Virtuoso window name.
fn window_mode(name: &str) -> &'static str {
    if name.contains("ADE Explorer Editing") || name.contains("ADE Assembler Editing") {
        "ade-editing"
    } else if name.contains("ADE Explorer Reading") {
        "ade-reading"
    } else if name.contains("ADE") {
        "ade-other"
    } else if name.contains("Schematic Editor") {
        "schematic"
    } else if name.contains("Layout Editor") {
        "layout"
    } else {
        "other"
    }
}

/// Parse the JSON string returned by list_windows().
///
/// SKILL encodes non-ASCII chars as octal escapes (e.g. `\256` = ®).
/// Standard JSON parsers reject these, so we decode them first.
fn parse_window_json(output: &str) -> Value {
    // Strip surrounding SKILL string quotes
    let s = output.trim_matches('"');
    // Decode SKILL octal escapes (\NNN) → UTF-8, then un-escape \" and \\
    let decoded = decode_skill_octal(s);
    let unescaped = decoded.replace("\\\"", "\"").replace("\\\\", "\\");
    serde_json::from_str(&unescaped).unwrap_or_else(|_| json!([]))
}

/// Convert SKILL's `\NNN` octal escapes to their UTF-8 codepoints.
/// Leaves other backslash sequences untouched (they are handled later).
fn decode_skill_octal(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
            // Collect up to 3 octal digits
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && end < start + 3 && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if let Ok(octal_str) = std::str::from_utf8(&bytes[start..end]) {
                if let Ok(n) = u32::from_str_radix(octal_str, 8) {
                    if let Some(c) = char::from_u32(n) {
                        out.push(c);
                        i = end;
                        continue;
                    }
                }
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn annotate_modes(v: Value) -> Value {
    match v {
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|mut item| {
                    if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                        let mode = window_mode(name).to_string();
                        item.as_object_mut()
                            .map(|o| o.insert("mode".into(), json!(mode)));
                    }
                    item
                })
                .collect(),
        ),
        other => other,
    }
}
