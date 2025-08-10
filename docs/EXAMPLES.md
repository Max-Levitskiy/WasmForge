# WasmForge Examples and Demos

This document provides runnable examples for common tasks using the WasmForge MCP server.

Prerequisites:
- Build the WASM module and desktop app first

Setup:
```bash
cd /Users/max/git/webtree/WasmForge/test-module
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
mkdir -p /Users/max/git/webtree/WasmForge/desktop-app/test-modules
cp target/wasm32-unknown-unknown/release/test_module.wasm ../desktop-app/test-modules/

cd /Users/max/git/webtree/WasmForge/desktop-app
cargo build
```

Run server (stdio):
```bash
cd /Users/max/git/webtree/WasmForge/desktop-app
./target/debug/desktop-app
```

- Press Ctrl+C to stop when done testing

## Tools List
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/debug/desktop-app
```
Expected (truncated):
```
{"tools":[{"name":"add","description":"Add two numbers ...","inputSchema":{...}}, ...]}
```

## Arithmetic: add
```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"add","arguments":{"a":5,"b":3}}}' | ./target/debug/desktop-app
```
Expected text contains: `WASM calculation result: 8`

## URL Validation
```bash
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"validate_url","arguments":{"url":"https://example.com"}}}' | ./target/debug/desktop-app
```
Expected text contains: `URL validation result: 1`

## Legacy Fetch (validate_url + process_response)
```bash
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"fetch","arguments":{"url":"https://httpbin.org/json"}}}' | ./target/debug/desktop-app
```
Expected text contains:
- `URL: https://httpbin.org/json`
- `Content (first 500 chars):`

## Web Browser: HTTP GET with WASM validation
```bash
echo '{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"prepare_http_get","arguments":{"url":"https://httpbin.org/json"}}}' | ./target/debug/desktop-app
```
Expected text contains:
- `HTTP GET successful!`
- `Content length: <N> bytes`
- `Content preview (first 500 chars):`

## File Ops: Read
```bash
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"prepare_file_read","arguments":{"path":"README.md"}}}' | ./target/debug/desktop-app
```
Expected text contains:
- `File read successful!`
- `Path: README.md`
- `Content length: <N> bytes`

## File Ops: Write
```bash
echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"prepare_file_write","arguments":{"path":"/tmp/wasmforge_demo.txt","content":"hello from wasmforge"}}}' | ./target/debug/desktop-app
```
Expected text contains:
- `File write successful!`
- `Path: /tmp/wasmforge_demo.txt`

## Shell Executor (allow-listed)
```bash
echo '{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"prepare_shell_exec","arguments":{"command":"echo hello"}}}' | ./target/debug/desktop-app
```
Expected text contains:
- `Shell execution completed.`
- `Exit code: 0`
- `STDOUT (truncated):\nhello`

## Recommend MCP Tools
```bash
echo '{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"prepare_recommend_mcps","arguments":{"task":"download a URL then save it to a file"}}}' | ./target/debug/desktop-app
```
Expected output: pretty-printed JSON array containing categories:
- `web_browser` with method `prepare_http_get`
- `file_ops` with methods `prepare_file_read`, `prepare_file_write`
- `shell_executor` (if relevant)

## TCP Mode Example
Run server:
```bash
cd /Users/max/git/webtree/WasmForge/desktop-app
./target/debug/desktop-app --port 8080
```
Query over TCP:
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | nc localhost 8080
```
Expected: same JSON as stdio `tools/list`.

Notes:
- Network-dependent outputs will vary.
- For `prepare_shell_exec`, allowed commands default to: echo, cat, ls, wc, uname. Configure via README section "Tool Security Allow-List".
