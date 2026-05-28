use crate::agent::mcp::protocol::{
    JsonRpcRequest, JsonRpcResponse, McpError, McpToolDef,
    METHOD_TOOLS_LIST, METHOD_TOOLS_CALL,
};
use crate::agent::mcp::transport::McpTransport;
use std::collections::HashMap;

/// Bridge between MCP servers and the agent's ToolRegistry.
///
/// Each MCP server gets an McpServer entry holding its transport and
/// the list of tools it provides.
pub struct McpBridge {
    servers: HashMap<String, McpServerEntry>,
    next_id: u64,
}

struct McpServerEntry {
    transport: Box<dyn McpTransport>,
    tools: Vec<McpToolDef>,
}

impl McpBridge {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register an MCP server with its transport.
    pub fn register_server(&mut self, name: &str, transport: Box<dyn McpTransport>) {
        self.servers.insert(
            name.to_string(),
            McpServerEntry {
                transport,
                tools: Vec::new(),
            },
        );
    }

    /// Discover tools from all registered MCP servers.
    /// Called once after registration to populate the tool list.
    pub fn discover_all_tools(&mut self) -> Result<(), String> {
        let server_names: Vec<String> = self.servers.keys().cloned().collect();
        for name in server_names {
            if let Err(e) = self.discover_tools(&name) {
                eprintln!("[mcp] Warning: failed to discover tools from '{}': {}", name, e);
            }
        }
        Ok(())
    }

    /// List tools from a single MCP server via tools/list.
    fn discover_tools(&mut self, server_name: &str) -> Result<(), String> {
        let entry = self
            .servers
            .get(server_name)
            .ok_or_else(|| format!("Server '{}' not registered", server_name))?;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: self.next_id(),
            method: METHOD_TOOLS_LIST.to_string(),
            params: serde_json::Value::Object(Default::default()),
        };

        let response = entry
            .transport
            .send(&request)
            .map_err(|e| format!("tools/list failed: {}", e))?;

        if let Some(err) = response.error {
            return Err(format!("MCP error {}: {}", err.code, err.message));
        }

        let result = response
            .result
            .ok_or_else(|| "tools/list returned no result".to_string())?;

        let tools: Vec<McpToolDef> = serde_json::from_value(
            result["tools"].clone(),
        )
        .unwrap_or_default();

        let entry_mut = self.servers.get_mut(server_name).unwrap();
        entry_mut.tools = tools;

        Ok(())
    }

    /// Call a tool on a specific MCP server.
    pub fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<String, McpError> {
        let entry = self
            .servers
            .get(server_name)
            .ok_or_else(|| McpError::Server(format!("Server '{}' not found", server_name)))?;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: self.next_id(),
            method: METHOD_TOOLS_CALL.to_string(),
            params: serde_json::json!({
                "name": tool_name,
                "arguments": arguments,
            }),
        };

        let response = entry.transport.send(&request)?;

        if let Some(err) = response.error {
            return Err(McpError::Server(format!(
                "MCP error {}: {}",
                err.code, err.message
            )));
        }

        let result = response
            .result
            .ok_or_else(|| McpError::Protocol("tools/call returned no result".into()))?;

        // Extract content from the result
        let content = result["content"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|c| c["text"].as_str())
                    .collect::<Vec<&str>>()
                    .join("\n")
            })
            .unwrap_or_else(|| result.to_string());

        Ok(content)
    }

    /// Get all tools from all servers (for ToolRegistry integration).
    pub fn get_all_tools(&self) -> Vec<(String, McpToolDef)> {
        let mut result = Vec::new();
        for (server_name, entry) in &self.servers {
            for tool in &entry.tools {
                result.push((server_name.clone(), tool.clone()));
            }
        }
        result
    }

    fn next_id(&self) -> u64 {
        // Simple incrementing ID — sufficient for single-agent use
        use std::sync::atomic::{AtomicU64, Ordering};
        static ID: AtomicU64 = AtomicU64::new(1);
        ID.fetch_add(1, Ordering::Relaxed)
    }
}
