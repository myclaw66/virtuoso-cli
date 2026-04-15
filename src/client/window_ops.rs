use crate::client::bridge::escape_skill_string;

pub struct WindowOps;

impl WindowOps {
    /// List all open Virtuoso windows.
    /// Returns a JSON array string: [{"name":"ADE Explorer Editing: ..."}]
    pub fn list_windows(&self) -> String {
        r#"let((out sep) out = "[" sep = "" foreach(w hiGetWindowList() out = strcat(out sep sprintf(nil "{\"name\":\"%s\"}" hiGetWindowName(w))) sep = ",") strcat(out "]"))"#.into()
    }

    /// Dismiss the current blocking dialog.
    /// action: "cancel" closes via Cancel; "ok" attempts OK/Yes button.
    pub fn dismiss_dialog(&self, action: &str) -> String {
        if action == "ok" {
            r#"let((d) d = hiGetCurrentDialog() if(d hiSendOK(d) "no-dialog"))"#.into()
        } else {
            r#"let((d) d = hiGetCurrentDialog() if(d hiCancelDialog(d) "no-dialog"))"#.into()
        }
    }

    /// Get the name of the current dialog without dismissing it.
    /// Returns "no-dialog" if no dialog is active.
    pub fn get_dialog_info(&self) -> String {
        r#"let((d) d = hiGetCurrentDialog() if(d hiGetWindowName(d) "no-dialog"))"#.into()
    }

    /// Capture a screenshot of the current Virtuoso window to a PNG file.
    ///
    /// IC23.1 does not have `hiGetWindowScreenDump`, so we use X11 `import`
    /// (ImageMagick) via system().  The file path is verified with `isFile`
    /// after the capture to distinguish success from failure.
    pub fn screenshot(&self, path: &str) -> String {
        let path = escape_skill_string(path);
        format!(
            r#"let((cmd) cmd = sprintf(nil "import -window root -display %s {path}" getShellEnvVar("DISPLAY")) system(cmd) if(isFile("{path}") "{path}" nil))"#
        )
    }

    /// Capture a screenshot of the first window whose name matches a regex pattern.
    /// Falls back to full-screen root capture (X11 import does not support per-window
    /// targeting without xdotool).
    pub fn screenshot_by_pattern(&self, path: &str, pattern: &str) -> String {
        let path = escape_skill_string(path);
        let pattern = escape_skill_string(pattern);
        format!(
            r#"let((matched cmd) matched = nil foreach(w hiGetWindowList() when(rexMatchp("{pattern}" hiGetWindowName(w)) matched = t)) if(matched let((cmd) cmd = sprintf(nil "import -window root -display %s {path}" getShellEnvVar("DISPLAY")) system(cmd) if(isFile("{path}") "{path}" nil)) "no-match"))"#
        )
    }
}
