# Tech Context

## Stack
- Rust 2024, Tokio, Wasmtime, Serde/JSON, Clap, Reqwest, TOML, Sha2, Dirs, UUID, md5
- Protocol: MCP JSON-RPC over stdio/TCP

## Build & Run (from README)
- Build WASM module: `cd test-module && cargo build --target wasm32-unknown-unknown --release` then copy to `desktop-app/test-modules/`
- Build server: `cd desktop-app && cargo build`
- Run (stdio): `./target/debug/desktop-app`
- Run (TCP): `./target/debug/desktop-app --port 8080`

## Test Commands (stdio)
- Tools list: `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/debug/desktop-app`
- Add: `echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"add","arguments":{"a":5,"b":3}}}' | ./target/debug/desktop-app`
- Fetch: `echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"fetch","arguments":{"url":"https://httpbin.org/json"}}}' | ./target/debug/desktop-app`
