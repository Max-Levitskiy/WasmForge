# Module Templates

This guide provides starter templates for creating new WasmForge modules in different languages and explains the ABI requirements so your exports are auto-discovered and callable as MCP tools.

- Supported templates now:
  - Rust (works today)
  - TypeScript via AssemblyScript (works today, with constraints)
  - Python (WASI) — documentation only for now; requires host WASI support

## ABI and Discovery Rules

- Exports must be plain C ABI functions with exact names.
- Exported WebAssembly memory must be available as `memory`.
- Supported function patterns:
  - i32_i32_to_i32: `fn add(a: i32, b: i32) -> i32`
  - ptr_len_to_i32: `fn validate_url(ptr: *const u8, len: usize) -> i32`
  - no_params_to_i32: `fn get_status() -> i32`
- The host writes input bytes at offset 1024 and then calls the export.
- No imports are provided (no WASI) in the current host execution path. Avoid requiring an allocator or external imports for handling inputs.

Tool discovery maps well-known names to useful MCP tools and schemas:
- `add` → arithmetic tool
- `validate_url`, `process_response` → enables virtual `fetch`
- `prepare_http_get`, `prepare_file_read`, `prepare_file_write`, `prepare_shell_exec`, `prepare_recommend_mcps` → specialized tools with schemas

## Rust Template

Folder: `templates/rust-basic/`

Build:
```bash
cd /Users/max/git/webtree/WasmForge/templates/rust-basic
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
mkdir -p /Users/max/git/webtree/WasmForge/desktop-app/test-modules
cp target/wasm32-unknown-unknown/release/module_template.wasm /Users/max/git/webtree/WasmForge/desktop-app/test-modules/
```

Notes:
- Exports use `extern "C"` and `#[unsafe(no_mangle)]` (Rust 2024).
- Reads input via raw pointer/length; avoid dynamic allocation.

## AssemblyScript (TypeScript) Template

Folder: `templates/assemblyscript-basic/`

Prereqs: Node.js >= 18

Build:
```bash
cd /Users/max/git/webtree/WasmForge/templates/assemblyscript-basic
npm install
npm run build:release
mkdir -p /Users/max/git/webtree/WasmForge/desktop-app/test-modules
cp build/assemblyscript_basic.wasm /Users/max/git/webtree/WasmForge/desktop-app/test-modules/
```

Notes:
- Uses `asc` with `--exportMemory` and `--runtime stub` to avoid a heavy runtime and allocations.
- Input is read directly from memory with `load<u8>(ptr + i)` to avoid heap usage.

## Python (WASI) Template — Planning

Folder: `templates/python-wasi/`

Status:
- Requires host WASI support (imports and context) before Python/WASI modules can run.
- Recommended path: CPython wasm32-wasi or Pyodide; expect larger binaries and slower startup.

Next steps to enable:
- Extend `WasmExecutor` to instantiate modules with WASI imports and a configured context.
- Confirm how to expose simple C-ABI exports that call into Python code or run a small embedded Python loop.

## Adding Your Module to Config

Edit the WasmForge config at `~/.config/wasmforge/config.toml` and add a new module entry:

```toml
[[modules]]
name = "my-module"
version = "0.1.0"
description = "My custom WASM module"
enabled = true

[modules.source]
type = "local"
path = "/Users/max/git/webtree/WasmForge/desktop-app/test-modules/my_module.wasm"
```

Restart the server and check discovery:
```bash
cd /Users/max/git/webtree/WasmForge/desktop-app
./target/debug/desktop-app
```

Then list tools and try calling them per `docs/EXAMPLES.md`.
