use serde_json::Value;
use std::sync::{Condvar, Mutex};

// ── Permission result ──

#[derive(Debug, Clone)]
pub enum PermissionResult {
    /// Tool call is safe, proceed without interruption.
    Allow,
    /// Tool call is blocked permanently.
    Deny(String),
    /// Tool call requires user approval via UI callback.
    AskUser {
        tool_name: String,
        args_preview: String,
        reason: String,
    },
}

// ── Deny list patterns (Gate 1) ──

const BASH_DENY_PATTERNS: &[&str] = &[
    "rm -rf /",
    "sudo ",
    "mkfs.",
    "dd if=",
    "> /dev/sda",
    "shutdown",
    "reboot",
    "chmod 777 /",
    ":(){ :|:& };:",
];

const FILE_DENY_PATTERNS: &[&str] = &[
    "/etc/passwd",
    "/etc/shadow",
    "/etc/sudoers",
    "/root/",
    "C:\\Windows\\System32",
    ".env.production",
    "credentials.json",
    "id_rsa",
    "id_ed25519",
];

// ── Rule matching (Gate 2) ──

/// Rules that trigger user approval (not outright denial).
fn check_rules(tool_name: &str, args: &Value) -> Option<String> {
    match tool_name {
        "write" | "edit" => {
            let path = args["file_path"].as_str().unwrap_or("");
            // Writing outside workspace-relative paths that look suspicious
            if path.starts_with('/') && !path.starts_with("/tmp/") {
                return Some(format!(
                    "Write to absolute path '{}' requires approval.",
                    path
                ));
            }
            // Writing to system config files
            if path.ends_with(".conf") || path.ends_with(".cfg") || path.ends_with(".ini") {
                return Some(format!(
                    "Write to config file '{}' requires approval.",
                    path
                ));
            }
        }
        "bash" => {
            let cmd = args["command"].as_str().unwrap_or("").to_lowercase();
            if cmd.contains("rm ") && !cmd.contains("rmdir") {
                return Some(format!(
                    "Delete command '{}' requires approval.",
                    args["command"].as_str().unwrap_or("")
                ));
            }
            if cmd.contains("chmod") || cmd.contains("chown") {
                return Some(format!(
                    "Permission change command requires approval."
                ));
            }
            if cmd.contains("pip install") || cmd.contains("npm install -g") {
                return Some(format!(
                    "Global package install requires approval."
                ));
            }
        }
        _ => {}
    }
    None
}

/// Gate 1: hard deny list check.
fn check_deny_list(tool_name: &str, args: &Value) -> Option<String> {
    match tool_name {
        "bash" => {
            let cmd = args["command"].as_str().unwrap_or("").to_lowercase();
            for pattern in BASH_DENY_PATTERNS {
                if cmd.contains(&pattern.to_lowercase()) {
                    return Some(format!(
                        "Blocked: command matches deny-list pattern '{}'",
                        pattern
                    ));
                }
            }
            if cmd.contains("; rm ") || cmd.contains("&& rm ") {
                return Some("Blocked: destructive chaining detected".into());
            }
        }
        "write" | "edit" | "read" => {
            let path = args["file_path"].as_str().unwrap_or("").to_lowercase();
            for pattern in FILE_DENY_PATTERNS {
                if path.contains(&pattern.to_lowercase()) {
                    return Some(format!(
                        "Blocked: path matches deny-list pattern '{}'",
                        pattern
                    ));
                }
            }
        }
        _ => {}
    }
    None
}

/// Main entry point: check permission for a tool call.
pub fn check_permission(tool_name: &str, args: &Value) -> PermissionResult {
    // Gate 1: DenyList
    if let Some(reason) = check_deny_list(tool_name, args) {
        return PermissionResult::Deny(reason);
    }

    // Gate 2: RuleMatch → trigger user approval
    if let Some(reason) = check_rules(tool_name, args) {
        return PermissionResult::AskUser {
            tool_name: tool_name.to_string(),
            args_preview: serde_json::to_string(args).unwrap_or_default(),
            reason,
        };
    }

    PermissionResult::Allow
}

// ── FFI permission callback synchronization ──

/// State shared between the Rust agent loop (waiter) and the FFI callback (signaller).
struct PermissionGate {
    /// Whether a permission request is pending.
    pending: bool,
    /// User response: true = allow, false = deny.
    allowed: bool,
    /// Tool name being asked about (for verification).
    request_id: String,
}

static PERMISSION_GATE: Mutex<PermissionGate> = Mutex::new(PermissionGate {
    pending: false,
    allowed: false,
    request_id: String::new(),
});

static PERMISSION_CONDVAR: Condvar = Condvar::new();

/// Called from the Rust agent loop (Gate 3): blocks until UI responds.
///
/// Returns true if the user approved, false if denied.
pub fn wait_for_user_approval(tool_name: &str, reason: &str) -> bool {
    let mut guard = PERMISSION_GATE.lock().unwrap();

    // Set up the request
    guard.pending = true;
    guard.allowed = false;
    guard.request_id = format!("{}:{}", tool_name, reason);

    // Call FFI callback to notify UI (if registered)
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(cb) = crate::ffi::load_permission_cb() {
            let tool_cstr = std::ffi::CString::new(tool_name).unwrap_or_default();
            let reason_cstr = std::ffi::CString::new(reason).unwrap_or_default();
            cb(tool_cstr.as_ptr(), reason_cstr.as_ptr());
        }
    }

    // Block until UI responds (with 30s timeout)
    let (mut guard, timeout) = PERMISSION_CONDVAR
        .wait_timeout(guard, std::time::Duration::from_secs(30))
        .unwrap();

    if timeout.timed_out() {
        guard.pending = false;
        return false; // Auto-deny on timeout
    }

    let result = guard.allowed;
    guard.pending = false;
    result
}

/// Called from FFI when the user clicks Allow/Deny in the UI.
///
/// # Safety
/// `allowed` must be 0 (deny) or 1 (allow).
pub fn resolve_permission(allowed: bool) {
    let mut guard = PERMISSION_GATE.lock().unwrap();
    guard.allowed = allowed;
    PERMISSION_CONDVAR.notify_one();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deny_list_bash_sudo() {
        let result = check_permission("bash", &serde_json::json!({"command": "sudo bash"}));
        match result {
            PermissionResult::Deny(reason) => assert!(reason.contains("sudo")),
            _ => panic!("expected Deny"),
        }
    }

    #[test]
    fn test_deny_list_file_passwd() {
        let result = check_permission(
            "read",
            &serde_json::json!({"file_path": "/etc/passwd"}),
        );
        match result {
            PermissionResult::Deny(reason) => assert!(reason.contains("passwd")),
            _ => panic!("expected Deny"),
        }
    }

    #[test]
    fn test_rule_ask_user_bash_rm() {
        let result = check_permission("bash", &serde_json::json!({"command": "rm file.txt"}));
        match result {
            PermissionResult::AskUser { reason, .. } => {
                assert!(reason.contains("Delete"));
            }
            _ => panic!("expected AskUser"),
        }
    }

    #[test]
    fn test_allow_safe_bash() {
        let result = check_permission("bash", &serde_json::json!({"command": "echo hello"}));
        match result {
            PermissionResult::Allow => {}
            _ => panic!("expected Allow for safe command"),
        }
    }

    #[test]
    fn test_resolve_permission() {
        // Simulate a permission resolution
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(50));
            resolve_permission(true);
        });

        let mut guard = PERMISSION_GATE.lock().unwrap();
        guard.pending = true;
        let (mut guard, timeout) = PERMISSION_CONDVAR
            .wait_timeout(guard, std::time::Duration::from_secs(5))
            .unwrap();
        assert!(!timeout.timed_out());
        assert!(guard.allowed);
        guard.pending = false;
    }
}
