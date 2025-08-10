# System Patterns

## Components
- Config: TOML config loading
- ModuleManager: module lifecycle and (future) caching/HTTP download
- WasmExecutor: Wasmtime engine, instance mgmt, function calls
- ToolDiscovery: map export signatures to MCP tool schemas
- WasmForgeServer: MCP JSON-RPC over stdio/TCP

## Flows
1. Startup: load config → load modules → discover tools → serve MCP
2. Tools/list: return discovered tool schemas
3. Tools/call: route to module export, translate params/results

## Patterns & Decisions
- Pattern-based signature mapping: `i32_i32_to_i32`, `ptr_len_to_i32`, `no_params_to_i32`
- TOML for human-friendly configuration
- Wasmtime for safety/perf; JSON-RPC MCP for interoperability
