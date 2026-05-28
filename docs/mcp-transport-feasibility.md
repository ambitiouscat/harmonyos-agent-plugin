# MCP Transport 可行性评估

## 背景

HmosAgent 需要在 HarmonyOS 上接入 MCP 协议以扩展工具能力。但由于 HarmonyOS App 沙箱禁止执行系统二进制文件（包括 shell、子进程），标准的 stdio transport 不可用。本报告评估替代方案。

## MCP Transport 选项对比

| Transport | HarmonyOS 可用？ | 说明 |
|-----------|-----------------|------|
| stdio | ❌ | 需要 `std::process::Command` 启动子进程，沙箱禁止 |
| StreamableHTTP | ⚠️ 部分 | 2025 MCP 规范草案新增，支持度有限 |
| WebSocket | ❌ | 需要 tokio，与同步底座冲突 |

## StreamableHTTP 支持度调研

截至 2026 年 5 月，StreamableHTTP 的生态支持：

| MCP 服务器 | HTTP Transport | 备注 |
|-----------|---------------|------|
| @modelcontextprotocol/server-filesystem | ❌ stdio only | 官方基础服务器 |
| @modelcontextprotocol/server-github | ❌ stdio only | 官方基础服务器 |
| @modelcontextprotocol/server-postgres | ❌ stdio only | 官方基础服务器 |
| 第三方 MCP 网关 (mcp-gateway) | ✅ | 可作为代理桥接 stdio→HTTP |

结论：**主流 MCP 服务器仍以 stdio 为主，HTTP transport 官方生态尚未成熟。**

## 推荐方案：Phase 11 范围调整

基于可行性评估，Phase 11 的执行策略调整为：

### 已完成 (11a) ✅
- JSON-RPC 数据模型 (protocol.rs)
- McpTransport trait (transport.rs)
- McpBridge 抽象 (bridge.rs)

### 延缓 (11b → 后续 Phase)
- UreqMcpTransport 实现 — 待生态成熟后再接入
- MCP 工具注册 — 暂无可用 MCP 服务器

### 替代路径
- 内置工具扩展：将常用 MCP-like 功能（filesystem、git）作为 Rust 内置工具实现
- 遵循 MCP 协议模型：内置工具使用与 MCP 相同的 JSON Schema 格式，未来切换 transport 时无需改接口

## 决策记录

- **日期**: 2026-05-29
- **决策**: P11b 延期，保留 11a 的 trait 抽象作为未来桩
- **理由**: HarmonyOS 沙箱 + MCP 生态 HTTP 支持不足，强行实现无法落地测试
