use crate::agent::platform::HostCapabilities;
use std::path::Path;
use std::path::PathBuf;
use std::thread;

/// HarmonyOS / desktop sync platform host using ureq + std::fs + std::thread.
pub struct HarmonyOSHost {
    sandbox_root: PathBuf,
    api_key: String,
    api_base_url: String,
}

impl HarmonyOSHost {
    pub fn new(sandbox_root: &str, api_key: &str, api_base_url: &str) -> Self {
        Self {
            sandbox_root: PathBuf::from(sandbox_root),
            api_key: api_key.to_string(),
            api_base_url: api_base_url.to_string(),
        }
    }

    pub fn sandbox_root(&self) -> &PathBuf {
        &self.sandbox_root
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn api_base_url(&self) -> &str {
        &self.api_base_url
    }

    /// Resolve a user-supplied path relative to the sandbox root,
    /// rejecting attempts to escape via `..` traversal.
    fn resolve_path(&self, path: &Path) -> Result<PathBuf, String> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.sandbox_root.join(path)
        };
        let canonical = candidate
            .canonicalize()
            .map_err(|e| format!("Invalid path '{}': {}", candidate.display(), e))?;
        let root_canonical = self
            .sandbox_root
            .canonicalize()
            .map_err(|e| format!("Sandbox root error: {}", e))?;
        if !canonical.starts_with(&root_canonical) {
            return Err(format!(
                "Sandbox violation: path '{}' is outside workspace",
                canonical.display()
            ));
        }
        Ok(canonical)
    }
}

impl HostCapabilities for HarmonyOSHost {
    fn read_file(&self, path: &Path) -> Result<String, String> {
        let resolved = self.resolve_path(path)?;
        std::fs::read_to_string(&resolved)
            .map_err(|e| format!("Failed to read '{}': {}", resolved.display(), e))
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), String> {
        let resolved = self.resolve_path(path)?;
        if let Some(parent) = resolved.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent dir: {}", e))?;
        }
        std::fs::write(&resolved, content)
            .map_err(|e| format!("Failed to write '{}': {}", resolved.display(), e))
    }

    fn http_post(&self, url: &str, body: &str) -> Result<String, String> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .into();

        let body_json: serde_json::Value = serde_json::from_str(body)
            .unwrap_or(serde_json::json!({}));

        let resp = agent
            .post(url)
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send_json(&body_json)
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = resp.status().as_u16();
        let resp_body = resp
            .into_body()
            .read_to_string()
            .unwrap_or_default();

        if status >= 400 {
            let err_detail = serde_json::from_str::<serde_json::Value>(&resp_body)
                .ok()
                .and_then(|v| v["error"]["message"].as_str().map(String::from))
                .unwrap_or_else(|| {
                    format!(
                        "HTTP {} — {}",
                        status,
                        &resp_body[..resp_body.len().min(300)]
                    )
                });
            return Err(err_detail);
        }

        Ok(resp_body)
    }

    fn execute_command(&self, cmd: &str) -> Result<String, String> {
        // Hard deny list for dangerous commands
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
        ];

        let cmd_lower = cmd.to_lowercase();
        for pattern in DENY_LIST {
            if cmd_lower.contains(pattern) {
                return Err(format!("Blocked: command matches deny pattern '{}'", pattern));
            }
        }

        // Use platform-appropriate shell.
        // Try /bin/sh as absolute path first for sandboxed environments (HarmonyOS),
        // fall back to "sh" for PATH lookup on standard Linux.
        #[cfg(target_os = "windows")]
        let shell = "cmd".to_string();
        #[cfg(all(not(target_os = "windows"), target_os = "linux"))]
        let shell = {
            if std::path::Path::new("/bin/sh").exists() {
                "/bin/sh".to_string()
            } else {
                "sh".to_string()
            }
        };
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        let shell = "sh".to_string();

        let flag = if shell.contains("cmd") { "/C" } else { "-c" };

        let output = std::process::Command::new(&shell)
            .arg(flag)
            .arg(cmd)
            .current_dir(&self.sandbox_root)
            .output()
            .map_err(|e| format!("Command execution failed: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            // Cap output at 50k chars
            if stdout.len() > 50_000 {
                let half = 25_000;
                let quarter = 12_500;
                Ok(format!(
                    "{}\n[... {} chars truncated ...]\n{}",
                    &stdout[..half],
                    stdout.len() - half - quarter,
                    &stdout[stdout.len() - quarter..]
                ))
            } else {
                Ok(stdout)
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(if stderr.is_empty() {
                format!("Command exited with status {}", output.status)
            } else {
                stderr
            })
        }
    }

    fn spawn_task(&self, f: Box<dyn FnOnce() + Send>) {
        thread::spawn(f);
    }
}
