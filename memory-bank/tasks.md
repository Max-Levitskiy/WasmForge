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

## P0 — OSS Readiness
- [x] Add dual licenses (MIT OR Apache-2.0) and update `Cargo.toml`
- [x] Switch to AGPL-3.0-only; remove MIT/Apache files; update manifests and docs
- [x] Add community health files: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`
- [x] Add GitHub Issue/PR templates
- [x] Add CI workflow (format, clippy, build wasm module and desktop app)
- [x] Add Dependabot for Cargo and Actions
- [x] Add `.editorconfig`
- [x] Add `rust-toolchain.toml` (nightly) to align with Rust 2024
- [x] Update `README.md` with Contributing and License sections
- [ ] Add badges once GitHub repo path is finalized

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
- [x] TCP mode example
- [x] Include expected output snippets in `docs/EXAMPLES.md` for each demo

## P0 — Module Template Suite (Rust, TypeScript, Python) — Plan (Level 3)

- Complexity level: Level 3 (multi-language scaffolding + ABI alignment; Python requires future WASI host support)

- Requirements analysis:
  - Export `memory` and functions that match WasmForge discovery patterns:
    - `i32_i32_to_i32` (e.g., `add(a: i32, b: i32) -> i32`)
    - `ptr_len_to_i32` (e.g., `validate_url(ptr,len) -> i32`, `prepare_http_get(ptr,len) -> i32`, etc.)
    - `no_params_to_i32` if needed
  - No imports expected by current host (no WASI/env). Instance is created with zero imports.
  - Host writes input bytes into module memory at offset 1024, then calls the export. Templates must be safe with raw pointer reads and avoid relying on managed allocators for incoming data.
  - Keep string/heap usage minimal to avoid static data near low memory; use byte-wise checks instead of string constants where practical.

- Components affected:
  - New `templates/` directory with subfolders:
    - `templates/rust-basic/` (works today)
    - `templates/assemblyscript-basic/` (TypeScript via AssemblyScript; works today with constraints)
    - `templates/python-wasi/` (placeholder + docs; requires future WASI host support)
  - Documentation: `docs/TEMPLATES.md` with interface expectations, build steps, and caveats.
  - `README.md` link to templates and quick-start build commands.

- Architecture considerations:
  - Rust: `no_std`-compatible cdylib style with `extern "C"` exports and `#[unsafe(no_mangle)]` for Rust 2024. No allocator required.
  - TypeScript: AssemblyScript with `--runtime stub` and `--exportMemory`; avoid heap/string constants; use `load<u8>(ptr+idx)` byte checks. Export the same function names used in examples for immediate compatibility.
  - Python: Realistic path is CPython/wasm32-wasi or Pyodide, which introduces WASI and imports. Our host must add WASI context and pass imports when instantiating. Until then, provide a template README and sample function signatures only.

- Implementation strategy:
  - Add scaffolding for Rust and AssemblyScript with minimal exports: `add`, `validate_url`, `prepare_http_get`, `prepare_file_read`, `prepare_file_write`, `prepare_shell_exec`, `prepare_recommend_mcps`.
  - Provide `build.sh` (or npm scripts) to emit `.wasm` artifacts and a local copy step to `desktop-app/test-modules/`.
  - Create `docs/TEMPLATES.md` explaining ABI requirements, pointer semantics, memory export, and build instructions.
  - For Python, add `templates/python-wasi/README.md` describing requirements and the host gaps to close (WASI support, imports, and potential size/perf tradeoffs).

- Detailed steps:
  1) Create `templates/rust-basic/` with `Cargo.toml` and `src/lib.rs` exporting the functions (mirroring `test-module` logic).
  2) Create `templates/assemblyscript-basic/` with `package.json`, `asconfig.json` (`--exportMemory`, `--runtime stub`), and `assembly/index.ts` using `load<u8>` for validation logic.
  3) Add `docs/TEMPLATES.md` covering interface patterns, example exports, and build & copy commands for both templates.
  4) Update `README.md` with a Templates section and quick-start commands.
  5) Verify: build both templates; ensure exported functions discovered by the host and pass existing EXAMPLES flows.
  6) Python: create `templates/python-wasi/README.md` documenting approach and host changes needed; do not integrate until WASI support exists.

- Dependencies:
  - Rust: `rustup target add wasm32-unknown-unknown`
  - AssemblyScript: Node.js >= 18; devDeps: `assemblyscript`, `asbuild` (or `asc`), configured `asconfig.json`
  - Python (future): Emscripten or CPython wasm32-wasi toolchain; Host WASI support via wasmtime

- Challenges & mitigations:
  - AssemblyScript memory layout conflicts with host writes at 1024: mitigate by avoiding heap and static strings; rely on byte-wise checks and stub runtime.
  - ABI mismatch: ensure `extern` exports with exact names and `(i32,i32)->i32` or `(i32,i32)->i32` patterns; verify with `tool_discovery` output.
  - Python/WASI not runnable today: track as creative-phase item and plan host changes (WASI context + imports).

- Creative phase components:
  - Python/WASI integration design: add WASI context in `WasmExecutor` and conditional instantiation with imports; evaluate size/performance and tool discovery mapping.

- Mode recommendation:
  - Proceed to IMPLEMENT MODE for Rust and TypeScript templates now.
  - Proceed to CREATIVE MODE for Python/WASI integration design before implementation.
