# WasmForge - WebAssembly MCP Platform

## Module Management System

WasmForge now features a comprehensive module management system that automatically discovers and exposes WebAssembly functions as MCP tools.

### Key Features

✅ **Dynamic Tool Discovery**: Automatically discovers exported WASM functions and creates MCP tools  
✅ **Configuration-Based Module Loading**: TOML configuration system for managing multiple modules  
✅ **Local and HTTP Module Sources**: Load modules from local files or remote URLs  
✅ **Integrity Verification**: SHA-256 checksums for downloaded modules  
✅ **Caching System**: Intelligent module caching with TTL support  
✅ **Multiple Module Support**: Load and manage multiple WASM modules simultaneously  
✅ **Pattern-Based Function Mapping**: Supports various WASM function signatures  

### Components

1. **desktop-app/**: Rust application with Wasmtime executor and MCP server
2. **test-module/**: Sample WASM module with `add(a, b)` and URL fetch capabilities

## Quick Start

### 1. Build the WASM Module
```bash
cd test-module
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/test_module.wasm ../desktop-app/test-modules/
```

### 2. Build the Desktop App
```bash
cd desktop-app
cargo build
```

### 3. Run the Server
```bash
# The first run will create a default configuration file
./target/debug/desktop-app
```

The server will:
- Create a configuration file at `~/.config/wasmforge/config.toml` (or platform equivalent)
- Load all enabled modules from the configuration
- Automatically discover and expose WASM functions as MCP tools
- Display a summary of discovered tools

### 4. Test the Tools
```bash
# Test tools list
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/debug/desktop-app

# Test addition
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"add","arguments":{"a":5,"b":3}}}' | ./target/debug/desktop-app

# Test URL fetch
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"fetch","arguments":{"url":"https://httpbin.org/json"}}}' | ./target/debug/desktop-app
```

See more runnable examples in `docs/EXAMPLES.md`.

## Configuration System

WasmForge uses a TOML configuration file for module management. Here's an example:

```toml
[server]
name = "wasmforge"
version = "0.1.0"
default_host = "127.0.0.1"

[[modules]]
name = "test-module"
version = "0.1.0"
description = "Test WebAssembly module with basic functions"
enabled = true

[modules.source]
type = "local"
path = "test-modules/test_module.wasm"

# Optional: custom tool configurations
[[modules.tools]]
name = "add"
description = "Add two numbers"
function_name = "add"

[cache]
directory = "/path/to/cache/wasmforge/modules"
max_size_mb = 100
ttl_hours = 24
```

### Module Sources

**Local Files:**
```toml
[modules.source]
type = "local"
path = "path/to/module.wasm"
```

**HTTP Downloads:**
```toml
[modules.source]
type = "http"
url = "https://example.com/module.wasm"
# Optional integrity check: SHA-256 hex string (no prefix)
checksum = "<sha256-hex>"
```

**Registry (Future):**
```toml
[modules.source]
type = "registry"
name = "my-module"
version = "1.0.0"
```

## Discovered Tools

The system automatically discovers WASM functions and creates appropriate MCP tools based on function signatures:

### Supported Function Patterns

1. **`i32_i32_to_i32`**: Functions taking two 32-bit integers and returning one
   - Example: `add(a: i32, b: i32) -> i32`
   - MCP Schema: `{"a": number, "b": number}`

2. **`ptr_len_to_i32`**: Functions taking a pointer and length (for string processing)
   - Example: `validate_url(ptr: *const u8, len: usize) -> i32`
   - MCP Schema: `{"data": string}`

3. **`no_params_to_i32`**: Functions taking no parameters and returning an integer
   - Example: `get_status() -> i32`
   - MCP Schema: `{}`

### Example Discovery Output
```
📋 Discovered Tools:
┌─────────────────────────────────────────────────────────────────┐
│ 📦 Module: test-module                                          │
├─────────────────────────────────────────────────────────────────┤
│ 🔧 add             Add two numbers using WebAssembly           │
│    └─ Pattern: i32_i32_to_i32                                  │
│ 🔧 validate_url    Validate URL format using WebAssembly       │
│    └─ Pattern: ptr_len_to_i32                                  │
│ 🔧 fetch           Fetch content from a URL with WASM validation│
│    └─ Pattern: ptr_len_to_i32                                  │
├─────────────────────────────────────────────────────────────────┤
└─────────────────────────────────────────────────────────────────┘
Total: 3 tools
```

## Server Modes

### Stdio Mode (Default)
```bash
./target/debug/desktop-app
```
Perfect for Claude Desktop integration.

### TCP Mode
```bash
# Local access
./target/debug/desktop-app --port 8080

# Remote access (use with caution)
./target/debug/desktop-app --port 8080 --host 0.0.0.0
```

### Test TCP Connection
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | nc localhost 8080
```

## Claude Desktop Integration

Add to your `~/.claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "wasmforge": {
      "command": "/path/to/wasmforge/desktop-app/target/debug/desktop-app"
    }
  }
}
```

After restarting Claude Desktop, you can use discovered tools:

```
Can you add 15 and 27 using the add tool?
```

```
Can you fetch the content from https://api.github.com/repos/anthropics/claude-code using the fetch tool?
```

## Advanced Features

### Module Caching
- Downloaded modules are cached locally with SHA-256 verification
- Cache TTL and size limits configurable
- Automatic cache validation and cleanup

### Security
- WASM sandboxing via Wasmtime
- URL validation for fetch operations
- Configurable resource limits (planned)

### Development Workflow
- Hot-reload modules during development (planned)
- Module templates for common patterns (planned)
- CLI tools for module management (planned)

## Architecture

The system consists of several key components:

- **`Config`**: TOML-based configuration management
- **`ModuleManager`**: Module loading, caching, and lifecycle management
- **`WasmExecutor`**: Multi-module WASM execution with Wasmtime
- **`ToolDiscovery`**: Automatic function discovery and MCP tool generation
- **`WasmForgeServer`**: MCP protocol server integration

## Security Notes

- When using TCP mode with `--host 0.0.0.0`, the server accepts connections from any IP
- Use firewall rules or VPN for secure remote access
- WASM modules run in a sandboxed environment but can make HTTP requests
- Always verify module checksums when downloading from external sources

### Tool Security Allow-List
The `prepare_shell_exec` tool is doubly validated: syntax/characters in WASM and an allow-list enforced by the host.

Defaults if unset: `echo`, `cat`, `ls`, `wc`, `uname`.

Configure via `config.toml`:

Structured security on tool:
```toml
[[modules.tools]]
name = "shell_executor"
description = "Execute simple shell commands"
function_name = "prepare_shell_exec"

[modules.tools.security]
allowed_commands = ["echo", "ls", "wc"]
```

Fallback metadata CSV on module:
```toml
[modules.metadata]
allowed_commands_csv = "echo,cat,ls,wc,uname"
```

The host resolves allowed commands in this priority: tool.security.allowed_commands -> modules.metadata.allowed_commands_csv -> defaults.

## Next Steps

This implementation provides a solid foundation for:
- Building a web-based module marketplace
- Creating a registry system for sharing WASM modules
- Adding more sophisticated WASM function patterns
- Implementing resource limits and advanced sandboxing
- Creating development tools and templates

## Contributing

Contributions are welcome! Please read `CONTRIBUTING.md` for setup and workflow.

We follow a `CODE_OF_CONDUCT.md` to ensure a welcoming community.

## License

Licensed under the GNU Affero General Public License v3.0 only (AGPL-3.0-only). See `LICENSE` for details.

## Module Templates

Starter templates and docs are available to create new modules:
- Rust: `templates/rust-basic/`
- TypeScript (AssemblyScript): `templates/assemblyscript-basic/`
- Python (WASI, planning): `templates/python-wasi/`

See `docs/TEMPLATES.md` for ABI expectations and step-by-step build instructions.
