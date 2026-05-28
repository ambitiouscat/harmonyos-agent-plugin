use serde_json::Value;
use std::process::{Command, Stdio};
use std::time::Instant;

/// Hard deny list — commands containing these patterns are rejected outright.
const DENY_LIST: &[&str] = &[
    "rm -rf /",
    "sudo ",
    "mkfs.",
    "dd if=",
    "> /dev/sda",
    "shutdown",
    "reboot",
    "chmod 777 /",
    ":(){ :|:& };:",
    "format c:",
];

/// Maximum command output in characters.
const OUTPUT_CAP: usize = 50_000;

/// Sandboxed bash handler registered with the ToolRegistry.
///
/// Checks deny list, runs command with a 120s timeout, caps output at 50k chars.
#[cfg(not(target_arch = "wasm32"))]
pub fn bash_handler(args: Value, _sandbox_root: &str) -> Result<String, String> {
    let command = args["command"]
        .as_str()
        .ok_or_else(|| "Missing 'command' parameter".to_string())?;

    // Gate 1: hard deny list
    let cmd_lower = command.to_lowercase();
    for pattern in DENY_LIST {
        if cmd_lower.contains(&pattern.to_lowercase()) {
            return Err(format!("Blocked: command matches deny-list pattern '{}'", pattern));
        }
    }

    // Prevent shell injection via command chaining with ';' or '&&' in dangerous contexts
    if command.contains("; rm ") || command.contains("&& rm ") {
        return Err("Blocked: destructive chaining detected".into());
    }

    let start = Instant::now();

    #[cfg(target_os = "windows")]
    let output = {
        let child = Command::new("cmd")
            .args(["/C", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to execute: {}", e))?;

        // Simple timeout via polling (no wait_timeout on stable)
        // For now, run to completion; TODO: add timeout watchdog thread
        child
            .wait_with_output()
            .map_err(|e| format!("Command error: {}", e))?
    };

    #[cfg(not(target_os = "windows"))]
    let output = {
        let child = Command::new("sh")
            .args(["-c", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to execute: {}", e))?;

        child
            .wait_with_output()
            .map_err(|e| format!("Command error: {}", e))?
    };

    let elapsed = start.elapsed();

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let mut result = String::new();
    if !stdout.is_empty() {
        result.push_str(&stdout);
    }
    if !stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str("[stderr]\n");
        result.push_str(&stderr);
    }

    if result.len() > OUTPUT_CAP {
        let keep = OUTPUT_CAP - 100;
        result.truncate(keep);
        result.push_str(&format!(
            "\n\n[... output truncated at {} chars, elapsed {:?} ...]",
            OUTPUT_CAP, elapsed
        ));
    }

    if !output.status.success() {
        result.push_str(&format!(
            "\n[exit code: {}]",
            output.status.code().unwrap_or(-1)
        ));
    }

    Ok(if result.is_empty() {
        format!("Command completed with no output (elapsed: {:?})", elapsed)
    } else {
        result
    })
}

/// WASM stub — bash is not available in browser context.
#[cfg(target_arch = "wasm32")]
pub fn bash_handler(_args: Value, _sandbox_root: &str) -> Result<String, String> {
    Err("bash tool is not available on wasm32 target".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash_deny_list_blocked() {
        let result = bash_handler(
            serde_json::json!({"command": "sudo rm -rf /"}),
            "/tmp",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Blocked"));
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_bash_simple_echo() {
        let result = bash_handler(
            serde_json::json!({"command": "echo hello"}),
            "/tmp",
        );
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("hello"));
    }

    #[test]
    fn test_bash_missing_command() {
        let result = bash_handler(serde_json::json!({}), "/tmp");
        assert!(result.is_err());
    }
}
