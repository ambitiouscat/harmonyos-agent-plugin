use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── JSON-RPC 2.0 data model ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// ── MCP error type ──

#[derive(Debug, Clone)]
pub enum McpError {
    Timeout(String),
    Transport(String),
    Protocol(String),
    Server(String),
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpError::Timeout(msg) => write!(f, "MCP timeout: {}", msg),
            McpError::Transport(msg) => write!(f, "MCP transport error: {}", msg),
            McpError::Protocol(msg) => write!(f, "MCP protocol error: {}", msg),
            McpError::Server(msg) => write!(f, "MCP server error: {}", msg),
        }
    }
}

// ── Standard MCP method names ──

pub const METHOD_TOOLS_LIST: &str = "tools/list";
pub const METHOD_TOOLS_CALL: &str = "tools/call";
pub const METHOD_RESOURCES_LIST: &str = "resources/list";
pub const METHOD_RESOURCES_READ: &str = "resources/read";
pub const METHOD_PROMPTS_LIST: &str = "prompts/list";
pub const METHOD_PROMPTS_GET: &str = "prompts/get";

// ── MCP tool descriptor (from tools/list response) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "inputSchema", default)]
    pub input_schema: Value,
}

// ── MCP server info (from initialize response) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "tools/list".into(),
            params: Value::Object(Default::default()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, 1);
        assert_eq!(parsed.method, "tools/list");
    }

    #[test]
    fn test_response_with_result() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"tools":[]}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_response_with_error() {
        let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_mcp_error_display() {
        assert!(McpError::Transport("test".into()).to_string().contains("transport"));
        assert!(McpError::Timeout("test".into()).to_string().contains("timeout"));
    }
}
