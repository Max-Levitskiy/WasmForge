# Project Brief

## Name
WasmForge — WebAssembly MCP Platform

## Purpose
Enable dynamic loading and execution of WebAssembly (WASM) modules as MCP tools, exposing WASM exports to LLM clients via the Model Context Protocol (stdio or TCP).

## Core Outcomes (Current Scope)
- Rust desktop MCP server with Wasmtime execution
- Dynamic tool discovery from WASM exports (pattern-based)
- TOML configuration for modules and server
- Test WASM module demonstrating `add`, URL validation, and fetch processing

## Out of Scope (for now)
- Cloud backend, registry, and web frontend
- Full module marketplace and analytics

## Success Criteria
- README flows work end-to-end
- Tools list and invocation succeed for test module
- Claude Desktop integration works via stdio
