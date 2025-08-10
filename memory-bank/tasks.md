# tasks.md — Single Source of Truth

## P0 — Iteration 1: Make default flow runnable (valuable + testable)
- [x] Align `validate_url` arg key with schema
  - Edit `desktop-app/src/main.rs`: in `ptr_len_to_i32` branch for `validate_url`, read `arguments["url"]` (not `data`) and update error messages accordingly.
- [x] Ensure local module path validates
  - Edit `desktop-app/src/config.rs`: in `validate()`, when `ModuleSource::Local { path }` is relative and missing at config dir, also check current working directory before erroring (match `ModuleManager` behavior).
- [x] Fix WASM export attributes for test module (Rust 2024)
  - Edit `test-module/src/lib.rs`: use `#[unsafe(no_mangle)]` for all exports (Rust 2024 requires marking `no_mangle` as unsafe).

### Build & Run
- [x] Build WASM module
  - `cd test-module && cargo build --target wasm32-unknown-unknown --release`
  - `mkdir -p desktop-app/test-modules && cp target/wasm32-unknown-unknown/release/test_module.wasm ../desktop-app/test-modules/`
- [x] Build desktop app
  - `cd desktop-app && cargo build`
- [x] Run (stdio)
  - From repo root: `./desktop-app/target/debug/desktop-app`

### Tests (MCP stdio)
- [x] List tools
  - `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./desktop-app/target/debug/desktop-app`
- [x] Call add
  - `echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"add","arguments":{"a":5,"b":3}}}' | ./desktop-app/target/debug/desktop-app`
- [x] Validate URL
  - `echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"validate_url","arguments":{"url":"https://example.com"}}}' | ./desktop-app/target/debug/desktop-app`
- [x] Fetch via legacy tool
  - `echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"fetch","arguments":{"url":"https://httpbin.org/json"}}}' | ./desktop-app/target/debug/desktop-app`
- [x] Read a safe file (prepare_file_read)
  - `echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"path":"README.md"}}' | ./desktop-app/target/debug/desktop-app`

### Acceptance Criteria
- [x] tools/list shows at least: `add`, `validate_url`, `process_response`, `prepare_http_get`, `prepare_file_read`, `prepare_file_write`, and virtual `fetch`.
- [x] add returns 8 in response text.
- [x] validate_url accepts `url` key and returns 1 for https URL.
- [x] fetch returns HTTP content preview without errors.
- [x] prepare_file_read succeeds for `README.md` and returns file content.

## P1 — Document and integrate
- [x] Record outcomes and any bugs in `progress.md`
- [x] Document MCP compliance observations
- [ ] Test Claude Desktop integration

## P2 — Next improvements
- [ ] Add HTTP module source support
- [ ] Implement module caching with checksum verification
- [ ] Expand function patterns and error handling

## P0 — Iteration 2: Add MCP tools for testing (shell, browser, recommend)
- [x] Shell Executor
  - WASM: add `prepare_shell_exec(ptr,len)->i32` in `test-module/src/lib.rs` to validate command text
    - Forbid newlines, pipes, redirection, `;`, `&&`, backticks, and length > 200
  - Config: make whitelist user-configurable
    - Preferred: extend `ToolConfig` with `security.allowed_commands: string[]`
    - Fallback (if avoiding schema change): use `ModuleConfig.metadata.allowed_commands_csv`
    - Default (if unset): `echo`, `cat`, `ls`, `wc`, `uname`
  - Host: add `execute_shell_with_validation(module_name: &str, command: &str) -> Result<String>` in `desktop-app/src/wasm_executor.rs`
    - Use `tokio::process::Command`; 10s timeout; capture stdout/stderr; return exit code
    - Enforce whitelist server-side using configured list (fallback to defaults)
  - Discovery: update `desktop-app/src/tool_discovery.rs` schema for `prepare_shell_exec` to `{ command: string }` with description "Execute a simple shell command with WASM validation"; optionally include "Allowed: ..." preview from config
  - Server: in `desktop-app/src/main.rs` `ptr_len_to_i32` branch, when `function_name == "prepare_shell_exec"`, call executor and return result text

- [x] Web Browser (GET)
  - Reuse existing `prepare_http_get` (already in test module) and `http_get_with_validation` host method
  - Ensure response includes URL, content length, and 500-char preview (already implemented)
  - Optionally refine description in discovery for clarity

- [x] Find Fit for Task (Recommendation)
  - WASM: add `prepare_recommend_mcps(ptr,len)->i32` in `test-module/src/lib.rs` to validate text query
    - Forbid overly long input (> 500 chars); allow alphanumerics and common punctuation
  - Host: add `recommend_mcps_with_validation(module_name: &str, query: &str) -> Result<String>` in `desktop-app/src/wasm_executor.rs`
    - If WASM returns 1, scan `ToolDiscovery` for matching tools by name/description keywords
    - Return JSON string with an array of recommended MCP tools, each with `name`, `description`, and `methods` (tool names + minimal input schemas)
    - Include at least: `shell_executor` (prepare_shell_exec), `web_browser` (prepare_http_get), `file_ops` (prepare_file_read/prepare_file_write)
  - Discovery: add schema for `prepare_recommend_mcps` as `{ task: string }`
  - Server: in `desktop-app/src/main.rs` `ptr_len_to_i32` branch, when `function_name == "prepare_recommend_mcps"`, call executor and return pretty JSON text

### Tests (MCP stdio)
- [x] Shell: echo
  - `echo '{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"prepare_shell_exec","arguments":{"command":"echo hello"}}}' | ./desktop-app/target/debug/desktop-app`
- [x] Browser: GET
  - `echo '{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"prepare_http_get","arguments":{"url":"https://httpbin.org/json"}}}' | ./desktop-app/target/debug/desktop-app`
- [x] Recommend: task suggestion
  - `echo '{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"prepare_recommend_mcps","arguments":{"task":"download a URL then save it to a file"}}}' | ./desktop-app/target/debug/desktop-app`

### Acceptance Criteria
- [x] `prepare_shell_exec` executes whitelisted commands and returns exit code + stdout/stderr summary; rejects unsafe input
- [x] `prepare_http_get` returns successful content preview for valid URLs and rejects invalid ones via WASM
- [x] `prepare_recommend_mcps` returns a JSON array including `shell_executor`, `web_browser`, and `file_ops` with method names and brief input schemas; items are relevant to query

## Examples and Demos
- [x] Create `docs/EXAMPLES.md` with runnable examples and expected outputs
- [x] Add README snippets for config-driven whitelist
- [x] Shell Executor demo commands (stdio)
- [x] Recommend MCPS demo (stdio)
- [x] File Ops demos (stdio)
- [ ] TCP mode example
- [x] Include expected output snippets in `docs/EXAMPLES.md` for each demo
