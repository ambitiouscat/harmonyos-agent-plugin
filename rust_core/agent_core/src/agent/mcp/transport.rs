use crate::agent::mcp::protocol::{JsonRpcRequest, JsonRpcResponse, McpError};

/// Abstraction for MCP transport — synchronous base + optional async extension.
///
/// HarmonyOS uses `send` (HTTP POST via ureq).
/// Desktop can also use `send_async` (stdio subprocess via tokio) when `feature = "async"` is enabled.
/// WASM returns not-supported errors.
pub trait McpTransport: Send + Sync {
    /// Send a JSON-RPC request synchronously and return the response.
    fn send(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse, McpError>;

    /// Whether this transport supports streaming responses.
    fn is_streamable(&self) -> bool {
        false
    }

    /// Send a JSON-RPC request asynchronously (only available with `feature = "async"`).
    #[cfg(feature = "async")]
    #[allow(unused_variables)]
    fn send_async(
        &self,
        request: JsonRpcRequest,
    ) -> Box<dyn std::future::Future<Output = Result<JsonRpcResponse, McpError>>> {
        unimplemented!("async transport not supported for this implementation")
    }
}
