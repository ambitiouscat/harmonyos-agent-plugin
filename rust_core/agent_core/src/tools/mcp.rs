use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    jsonrpc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

/// Synchronous MCP client using std::net::TcpStream.
///
/// MCP (Model Context Protocol) servers are spawned as child processes
/// and communicate over stdio (stdin/stdout). This client supports
/// the basic MCP lifecycle: initialize, list_tools, call_tool.
pub struct McpBridge {
    servers: HashMap<String, McpServerConfig>,
}

struct McpServerConfig {
    server: McpServer,
    process: Option<std::process::Child>,
}

impl McpBridge {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    pub fn register_server(&mut self, server: McpServer) {
        self.servers.insert(
            server.name.clone(),
            McpServerConfig {
                server,
                process: None,
            },
        );
    }

    pub fn start_server(&mut self, name: &str) -> Result<(), String> {
        let config = self
            .servers
            .get_mut(name)
            .ok_or_else(|| format!("Server '{}' not registered", name))?;

        let child = std::process::Command::new(&config.server.command)
            .args(&config.server.args)
            .envs(&config.server.env)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP server '{}': {}", name, e))?;

        config.process = Some(child);
        Ok(())
    }

    pub fn call_tool(
        &mut self,
        server_name: &str,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let config = self
            .servers
            .get_mut(server_name)
            .ok_or_else(|| format!("Server '{}' not registered", server_name))?;

        let child = config
            .process
            .as_mut()
            .ok_or_else(|| format!("Server '{}' not started", server_name))?;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "tools/call".into(),
            params: Some(serde_json::json!({
                "name": tool_name,
                "arguments": arguments,
            })),
        };

        let request_json =
            serde_json::to_string(&request).map_err(|e| format!("JSON error: {}", e))?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or("Cannot access stdin")?;
        stdin
            .write_all(request_json.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        stdin
            .write_all(b"\n")
            .map_err(|e| format!("Write error: {}", e))?;
        stdin
            .flush()
            .map_err(|e| format!("Flush error: {}", e))?;

        let stdout = child
            .stdout
            .as_mut()
            .ok_or("Cannot access stdout")?;
        let mut buf = String::new();
        stdout
            .read_to_string(&mut buf) // MCP uses newline-delimited JSON
            .map_err(|e| format!("Read error: {}", e))?;

        let response: JsonRpcResponse =
            serde_json::from_str(&buf).map_err(|e| format!("Parse error: {}", e))?;

        if let Some(error) = response.error {
            Err(format!("MCP error: {}", error))
        } else if let Some(result) = response.result {
            Ok(result)
        } else {
            Err("Empty MCP response".into())
        }
    }

    pub fn list_tools(&mut self, server_name: &str) -> Result<Vec<McpTool>, String> {
        let config = self
            .servers
            .get_mut(server_name)
            .ok_or_else(|| format!("Server '{}' not registered", server_name))?;

        let child = config
            .process
            .as_mut()
            .ok_or_else(|| format!("Server '{}' not started", server_name))?;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "tools/list".into(),
            params: None,
        };

        let request_json =
            serde_json::to_string(&request).map_err(|e| format!("JSON error: {}", e))?;

        let stdin = child.stdin.as_mut().ok_or("Cannot access stdin")?;
        stdin
            .write_all(request_json.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        stdin.write_all(b"\n").map_err(|e| format!("Write error: {}", e))?;
        stdin.flush().map_err(|e| format!("Flush error: {}", e))?;

        let stdout = child.stdout.as_mut().ok_or("Cannot access stdout")?;
        let mut buf = String::new();
        stdout
            .read_to_string(&mut buf)
            .map_err(|e| format!("Read error: {}", e))?;

        let response: JsonRpcResponse =
            serde_json::from_str(&buf).map_err(|e| format!("Parse error: {}", e))?;

        if let Some(result) = response.result {
            let tools: Vec<McpTool> = result
                .get("tools")
                .and_then(|t| serde_json::from_value(t.clone()).ok())
                .unwrap_or_default();
            Ok(tools)
        } else {
            Err("No result in MCP response".into())
        }
    }

    pub fn stop_server(&mut self, name: &str) -> Result<(), String> {
        if let Some(config) = self.servers.get_mut(name) {
            if let Some(ref mut child) = config.process {
                let _ = child.kill();
                let _ = child.wait();
            }
            config.process = None;
        }
        Ok(())
    }
}

impl Default for McpBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for McpBridge {
    fn drop(&mut self) {
        let names: Vec<String> = self.servers.keys().cloned().collect();
        for name in names {
            let _ = self.stop_server(&name);
        }
    }
}
