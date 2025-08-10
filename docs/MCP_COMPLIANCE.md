# MCP Compliance Notes

This document summarizes the current MCP (Model Context Protocol) implementation and any known deviations.

## Supported Methods
- initialize
- tools/list
- tools/call

## initialize
Request:
```json
{"jsonrpc":"2.0","id":1,"method":"initialize"}
```
Response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": { "tools": {} },
    "serverInfo": { "name": "wasmforge", "version": "0.1.0" }
  }
}
```

Notes:
- `protocolVersion` is currently set to "2024-11-05" to align with recent MCP drafts.

## tools/list
Request:
```json
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
```
Response (shape):
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "add",
        "description": "...",
        "inputSchema": {"type":"object", "properties": { ... }}
      }
    ]
  }
}
```

- `inputSchema` contains the JSON schema for tool arguments.

## tools/call
Request (example):
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "add",
    "arguments": {"a": 5, "b": 3}
  }
}
```
Response (shape):
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {"type": "text", "text": "WASM calculation result: 8 (from test-module::add)"}
    ]
  }
}
```

- Responses return `content` as an array of parts. Text output is provided as a single `text` part.
- Error responses follow JSON-RPC error shape with `code` and `message`.

## Known Deviations / Clarifications
- The server focuses on the Tools capability; other MCP capabilities are not implemented.
- `tools/list` returns a `tools` array directly in `result` for simplicity.
- Tool output is returned as plaintext inside the `text` content part; structured outputs (JSON) are embedded as strings when applicable.

## Recommendations
- When integrating with clients, handle `content` arrays by concatenating `text` items.
- Validate tool schemas from `tools/list` before constructing `tools/call` requests.

