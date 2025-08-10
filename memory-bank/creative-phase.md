# 🎨🎨🎨 ENTERING CREATIVE PHASE

## Component: Shell Executor (MCP tool via WASM validation)

### Requirements & Constraints
- Validate shell command in WASM (`prepare_shell_exec(ptr,len)->i32`)
- Host executes with strong safeguards: user-configurable command whitelist (with safe defaults), 10s timeout, capture stdout/stderr, return exit code
- Schema in discovery: `{ command: string }`
- Deny: newlines, pipes, redirection, `;`, `&&`, backticks, >200 chars

### Options
1) Dual validation (WASM + host) with config-driven whitelist and timeout
- Pros: Defense in depth, configurable per deployment, least privilege
- Cons: Slight complexity; requires config plumbing

2) WASM-only validation; host trusts module
- Pros: Simpler host; flexible per-module policies
- Cons: Security risk if module is compromised; not acceptable

3) Host-only validation; ignore WASM
- Pros: Centralized policy; simpler module
- Cons: Loses sandbox benefit; coupling to host rules

### Recommendation
- Option 1: Dual validation. Use a config-driven whitelist (global + per-tool override) with safe defaults if unset: `echo`, `cat`, `ls`, `wc`, `uname`. Enforce max length 200. Timebox to 10s. Capture stdout/stderr and exit code.

### Implementation Guidelines
- Config (in `desktop-app/src/config.rs`)
  - Extend `ToolConfig` to support an optional security field, e.g.:
    - `security?: { allowed_commands?: string[] }`
    - Backward-compatible interim alternative: use `ModuleConfig.metadata` key `allowed_commands_csv` (comma-separated), but prefer structured field above.
  - Example TOML snippet:
    - For structured field (preferred):
      ```toml
      [[modules.tools]]
      name = "shell_executor"
      description = "Execute a simple shell command"
      function_name = "prepare_shell_exec"
      
      [modules.tools.security]
      allowed_commands = ["echo", "ls", "wc"]
      ```
    - For metadata fallback:
      ```toml
      [modules.metadata]
      allowed_commands_csv = "echo,ls,wc"
      ```
- WASM (in `test-module/src/lib.rs`)
  - Add `#[no_mangle] pub extern "C" fn prepare_shell_exec(ptr: *const u8, len: usize) -> i32`
  - Validate: ASCII; allowed chars `[A-Za-z0-9._/ -]`; no `|`, `>`, `<`, `;`, `&`, `\n`, backticks; length ≤ 200
  - Return 1 if valid; else 0 (do not hardcode command list in WASM)
- Discovery (in `desktop-app/src/tool_discovery.rs`)
  - Recognize function name `prepare_shell_exec` as pattern `ptr_len_to_i32`
  - Input schema: `{ command: string }` with description "Execute a simple shell command with WASM validation"
  - Optionally append configured whitelist preview to description (e.g., "Allowed: echo, ls, wc")
- Executor (in `desktop-app/src/wasm_executor.rs`)
  - Add `pub async fn execute_shell_with_validation(module_name: &str, command: &str) -> Result<String>`
  - Steps:
    - Call `prepare_shell_exec` via `call_function_ptr_len_to_i32`; require 1
    - Resolve allowed commands: per-tool security.allowed_commands → module metadata fallback → default `["echo","cat","ls","wc","uname"]`
    - Parse first token; must be in resolved whitelist
    - Use `tokio::process::Command` with `kill_on_drop(true)`; set 10s timeout via `tokio::time::timeout`
    - Disallow shell interpretation: invoke binary directly, split by spaces, no shell
    - Capture stdout/stderr; truncate to 4KB each
    - Return JSON-ish text: `{exit_code, stdout_preview, stderr_preview}`
- Server (in `desktop-app/src/main.rs`)
  - In `ptr_len_to_i32` branch: if `function_name == "prepare_shell_exec"`, read `arguments["command"]`, call executor, return text

### Verification
- Valid commands run and return exit code + previews
- Unsafe input rejected by both WASM and host
- Changing config updates allowed commands without rebuild
- Timeout enforced; large output truncated

---

## Component: MCP Tool Recommendation (Find Fit for Task)

### Requirements & Constraints
- WASM export: `prepare_recommend_mcps(ptr,len)->i32` validates free-text task (≤500 chars)
- Host scans discovered tools and returns JSON array with `{ name, description, methods }`
- Include at least `shell_executor`, `web_browser`, `file_ops` families when relevant
- No external network/ML; pure local heuristic ranking

### Options (Algorithm)
1) Keyword heuristic with scoring
- Tokenize task; compare against tool names/descriptions/schemas
- Pros: Simple, fast (O(T * V)); explainable
- Cons: Requires curated keyword sets; may miss semantics

2) TF-IDF cosine similarity (on-the-fly corpus)
- Pros: Better weighting; robust to noise
- Cons: More code; still shallow semantics; per-run IDF unstable with tiny corpora

3) Fuzzy string matching (edit distance) on tokens and n-grams
- Pros: Handles typos; simple
- Cons: Weak semantics; can overfit short names

### Recommendation
- Hybrid: Option 1 with light fuzzy boosts.
  - Base score: sum of keyword hits with field weights (name>desc>schema)
  - Boost exact phrase matches; penalize stop-words; small Levenshtein boost for close matches
  - Ensure deterministic top-K (e.g., K=5) with tie-break by name

### Implementation Guidelines
- WASM (`test-module/src/lib.rs`)
  - Add `#[no_mangle] pub extern "C" fn prepare_recommend_mcps(ptr: *const u8, len: usize) -> i32`
  - Validate ASCII/UTF-8; length ≤ 500; allow letters, numbers, spaces, common punctuation
- Executor (`desktop-app/src/wasm_executor.rs`)
  - Add `pub fn recommend_mcps_with_validation(&mut self, module_name: &str, query: &str, tools: &ToolDiscovery) -> Result<String>`
  - Steps:
    - Call `prepare_recommend_mcps`; require 1
    - Build index of tools: name, description, schema text, and method list by module
    - Scoring: keyword_overlap*weights + phrase_match + fuzzy_boost; normalize
    - Construct output JSON string with array of up to 5 items: `{ name, description, methods: [ { name, inputSchemaSummary } ] }`
- Discovery (`desktop-app/src/tool_discovery.rs`)
  - Recognize `prepare_recommend_mcps` as pattern `ptr_len_to_i32` with schema `{ task: string }`
- Server (`desktop-app/src/main.rs`)
  - In `ptr_len_to_i32` branch: when `function_name == "prepare_recommend_mcps"`, call executor helper with `server.tool_discovery` reference; pretty-print JSON

### Verification
- Query "download a URL then save it to a file" returns `web_browser` and `file_ops` at top, with correct methods
- Input >500 chars rejected
- Deterministic results; tie-break stable

---

## Component: Tool Discovery UX/Schema Refinements

### Requirements
- Clarify descriptions; align `validate_url` input key to `url`
- Ensure `fetch` legacy tool remains for backward compatibility

### Options
1) Keep minimal schemas; improve descriptions only
2) Expand schemas with constraints (regex/length) where obvious
3) Namespacing tool names consistently beyond `test-module`

### Recommendation
- Do 1 + light of 2: Improve descriptions; add basic field descriptions and required lists (already done in many places). Keep current namespacing: for non-`test-module`, prefix with module.

### Implementation Guidelines
- Update `tool_discovery.rs` schema for known ptr/len functions to include helpful descriptions; ensure `validate_url` tool schema uses `{ url: string }`
- In `main.rs`, when handling `validate_url`, read `arguments["url"]`

### Verification
- `tools/list` shows clear descriptions
- `validate_url` accepts `url` key and works

---

## Cross-Cutting Security Considerations
- Enforce strict input validation in WASM and host
- Timeouts for external operations (HTTP, shell)
- Output truncation to prevent flooding
- No shell parsing; invoke binaries directly

---

## Acceptance Mapping (to tasks.md)
- Shell Executor: P0 Iteration 2 requirements satisfied
- Web Browser: reuse `prepare_http_get` path; ensure response formatting
- Recommend MCPS: returns JSON array including `shell_executor`, `web_browser`, `file_ops`
- Validate URL: align argument key to `url`

---

## Verification Checkpoint
- Multiple options explored per component
- Pros/cons analyzed
- Recommendations justified against security and simplicity
- Concrete implementation guidelines provided

# 🎨🎨🎨 EXITING CREATIVE PHASE
