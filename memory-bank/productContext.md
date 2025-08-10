# Product Context

## Why
- Provide a safe, fast, language-agnostic way to add capabilities to MCP via WASM.

## Who
- Developers building MCP tools; users of Claude Desktop/other MCP clients.

## How It Should Work (UX)
- Start the desktop app, it discovers tools from configured WASM modules.
- MCP client lists tools and calls them with JSON arguments.
- Minimal setup: build test module, run desktop app, test via stdio or TCP.

## Key Value
- Security via sandboxing, portability, easy distribution, and automatic tool discovery.
