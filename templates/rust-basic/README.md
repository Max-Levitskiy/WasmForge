# Rust Module Template

This template produces a `.wasm` module compatible with WasmForge's discovery and execution.

Build and copy:
```bash
cd /Users/max/git/webtree/WasmForge/templates/rust-basic
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
mkdir -p /Users/max/git/webtree/WasmForge/desktop-app/test-modules
cp target/wasm32-unknown-unknown/release/module_template.wasm /Users/max/git/webtree/WasmForge/desktop-app/test-modules/
```

Exports:
- `add(a: i32, b: i32) -> i32`
- `validate_url(ptr,len) -> i32`
- `process_response(ptr,len) -> i32`
- `prepare_http_get(ptr,len) -> i32`
- `prepare_file_read(ptr,len) -> i32`
- `prepare_file_write(ptr,len) -> i32`
- `prepare_shell_exec(ptr,len) -> i32`
- `prepare_recommend_mcps(ptr,len) -> i32`

Notes:
- Uses `extern "C"` and `#[unsafe(no_mangle)]` (Rust 2024).
- Host writes input bytes to `memory` at offset 1024.
- Avoid relying on allocators for incoming data; read via raw pointers.

