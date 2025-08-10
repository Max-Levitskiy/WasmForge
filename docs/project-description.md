# Complete Implementation Plan: WasmForge - WebAssembly MCP Platform

## Repository Structure & High-Level Architecture

### Repository Organization
```
WasmForge/
├── desktop-app/           # Rust desktop application
├── platform/              # Backend + Frontend (TypeScript)
│   ├── backend/           # Express + Drizzle backend
│   ├── frontend/          # React + shadcn frontend
│   └── shared/            # Shared types and utilities
├── cli-tool/              # Command-line interface
├── templates/             # MCP server templates
│   ├── rust-template/
│   ├── typescript-template/
│   └── python-template/
└── examples/              # Sample implementations
```

### Technology Stack Summary
- **Desktop App**: Rust + Tauri + Wasmtime
- **Backend**: TypeScript + Bun + Express + Drizzle + Supabase
- **Frontend**: React + shadcn/ui + Tailwind + Framer Motion
- **CLI**: Rust (shared core with desktop app)
- **Templates**: Multi-language with standardized build targets

## Component Specifications

### 1. Desktop App (`desktop-app/`)
**Core Responsibilities:**
- WebAssembly module execution via Wasmtime
- Single MCP JSON-RPC endpoint on configurable port
- Module download and caching from S3
- Credential management via `.env` files
- Module lifecycle management (install/update/remove)

### 2. Backend (`platform/backend/`)
**Core Responsibilities:**
- User authentication via Supabase Auth
- Module registry with metadata storage
- S3 presigned URL generation for module distribution
- Build pipeline coordination for submitted modules
- API serving for frontend and desktop app

### 3. Frontend (`platform/frontend/`)
**Core Responsibilities:**
- Module marketplace browsing and search
- Module submission and management interface
- User dashboard with installed modules
- Documentation and template downloads

### 4. Templates (`templates/`)
**Standardized Structure:**
- Boilerplate code for each language
- Common build scripts targeting WASM
- MCP protocol integration helpers
- Example tool implementations

### 5. CLI Tool (`cli-tool/`)
**Core Responsibilities:**
- Module development workflow commands
- Local testing and debugging
- Template initialization
- Desktop app management

## Detailed Desktop App Implementation Plan

### Phase 1: Core WASM Executor (Week 1-2)

#### Project Setup
```bash
# Create Rust project with required dependencies
cargo new wasmforge-desktop --bin
cd wasmforge-desktop

# Add to Cargo.toml
[dependencies]
wasmtime = "25.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
sha2 = "0.10"
dirs = "5.0"
```

#### Core Architecture
```rust
// src/main.rs - Entry point
mod wasm_executor;
mod mcp_server;
mod module_manager;
mod config;

use crate::mcp_server::MCPServer;

#[tokio::main]
async fn main() -> anyhow::Result {
    let config = config::load_config()?;
    let server = MCPServer::new(config).await?;
    server.run().await
}
```

```rust
// src/wasm_executor.rs - WASM module execution
use wasmtime::*;

pub struct WasmExecutor {
    engine: Engine,
    modules: HashMap,
    instances: HashMap,
}

impl WasmExecutor {
    pub fn new() -> Result {
        let engine = Engine::default();
        Ok(Self {
            engine,
            modules: HashMap::new(),
            instances: HashMap::new(),
        })
    }

    pub async fn load_module(&mut self, name: &str, wasm_bytes: &[u8]) -> Result {
        let module = Module::new(&self.engine, wasm_bytes)?;
        self.modules.insert(name.to_string(), module);
        Ok(())
    }

    pub async fn call_tool(&mut self, module_name: &str, tool_name: &str, params: serde_json::Value) -> Result {
        // Implementation for calling WASM exported functions
        todo!()
    }
}
```

```rust
// src/mcp_server.rs - MCP JSON-RPC server
use serde_json::{Value, json};
use tokio::net::{TcpListener, TcpStream};

pub struct MCPServer {
    executor: WasmExecutor,
    config: Config,
    modules: Vec,
}

impl MCPServer {
    pub async fn new(config: Config) -> Result {
        let executor = WasmExecutor::new()?;
        Ok(Self {
            executor,
            config,
            modules: Vec::new(),
        })
    }

    pub async fn run(&self) -> Result {
        let addr = format!("127.0.0.1:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        
        println!("WasmForge MCP Server listening on {}", addr);
        
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(self.handle_connection(stream));
        }
        
        Ok(())
    }

    async fn handle_connection(&self, stream: TcpStream) {
        // Handle MCP JSON-RPC protocol
        todo!()
    }
}
```

### Phase 2: MCP Protocol Implementation (Week 2-3)

#### MCP Message Handling
```rust
// src/mcp_protocol.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: Option,
    pub method: String,
    pub params: Option,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: Option,
    pub result: Option,
    pub error: Option,
}

impl MCPServer {
    async fn handle_mcp_request(&mut self, request: MCPRequest) -> MCPResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,
            _ => MCPResponse::error(request.id, -32601, "Method not found"),
        }
    }

    async fn handle_tools_call(&mut self, request: MCPRequest) -> MCPResponse {
        let params = request.params.unwrap_or_default();
        let tool_name = params["name"].as_str().unwrap();
        let arguments = &params["arguments"];
        
        // Route to appropriate WASM module
        match self.executor.call_tool("module_name", tool_name, arguments.clone()).await {
            Ok(result) => MCPResponse::success(request.id, result),
            Err(e) => MCPResponse::error(request.id, -32000, &e.to_string()),
        }
    }
}
```

### Phase 3: Module Management (Week 3-4)

#### Module Download and Caching
```rust
// src/module_manager.rs
use std::path::PathBuf;
use sha2::{Sha256, Digest};

pub struct ModuleManager {
    cache_dir: PathBuf,
    client: reqwest::Client,
}

impl ModuleManager {
    pub fn new() -> Result {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wasmforge")
            .join("modules");
        
        std::fs::create_dir_all(&cache_dir)?;
        
        Ok(Self {
            cache_dir,
            client: reqwest::Client::new(),
        })
    }

    pub async fn download_module(&self, module_id: &str, presigned_url: &str) -> Result> {
        let module_path = self.cache_dir.join(format!("{}.wasm", module_id));
        
        if module_path.exists() {
            return Ok(std::fs::read(&module_path)?);
        }
        
        let response = self.client.get(presigned_url).send().await?;
        let bytes = response.bytes().await?;
        
        // Verify integrity
        let hash = Sha256::digest(&bytes);
        // TODO: Verify against expected hash from backend
        
        std::fs::write(&module_path, &bytes)?;
        Ok(bytes.to_vec())
    }
}
```

## Minimal Proof of Concept Implementation

### Step 1: Basic WASM Hello World

**Create Test WASM Module (Rust):**
```rust
// test-module/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Build Script:**
```bash
# test-module/build.sh
wasm-pack build --target nodejs --out-dir pkg
cp pkg/test_module.wasm ../desktop-app/test-modules/
```

### Step 2: Desktop App Minimal Version

```rust
// src/main.rs - Minimal test version
use wasmtime::*;
use std::fs;

fn main() -> anyhow::Result {
    let engine = Engine::default();
    let module_bytes = fs::read("test-modules/test_module.wasm")?;
    let module = Module::new(&engine, &module_bytes)?;
    
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])?;
    
    // Test calling WASM function
    let hello_func = instance.get_typed_func::(&mut store, "add")?;
    let result = hello_func.call(&mut store, (5, 3))?;
    
    println!("WASM add(5, 3) = {}", result);
    
    // Start basic MCP server
    start_mcp_server()?;
    
    Ok(())
}

fn start_mcp_server() -> anyhow::Result {
    println!("WasmForge MCP Server started on stdio");
    
    // Basic stdio MCP server for testing with Claude Desktop
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if let Ok(request) = serde_json::from_str::(&input) {
            let response = handle_mcp_message(request);
            println!("{}", serde_json::to_string(&response)?);
        }
    }
}

fn handle_mcp_message(request: serde_json::Value) -> serde_json::Value {
    let method = request["method"].as_str().unwrap_or("");
    
    match method {
        "initialize" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": request["id"],
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "wasmforge",
                    "version": "0.1.0"
                }
            }
        }),
        "tools/list" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": request["id"],
            "result": {
                "tools": [
                    {
                        "name": "add",
                        "description": "Add two numbers using WASM",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "a": {"type": "number"},
                                "b": {"type": "number"}
                            },
                            "required": ["a", "b"]
                        }
                    }
                ]
            }
        }),
        "tools/call" => {
            // Call WASM function and return result
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": request["id"],
                "result": {
                    "content": [
                        {
                            "type": "text",
                            "text": "WASM calculation result: 8"
                        }
                    ]
                }
            })
        },
        _ => serde_json::json!({
            "jsonrpc": "2.0",
            "id": request["id"],
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        })
    }
}
```

### Step 3: Claude Desktop Integration Test

**Claude Desktop Configuration:**
```json
// ~/.claude_desktop_config.json
{
  "mcpServers": {
    "wasmforge": {
      "command": "path/to/wasmforge-desktop",
      "args": ["--test-mode"]
    }
  }
}
```

## Testing Strategy

### Phase 1 Tests
1. **WASM Loading**: Verify modules load and functions execute
2. **Basic MCP**: Test initialize/tools/list/tools/call flow
3. **Claude Integration**: Confirm Claude Desktop can connect and call tools

### Phase 2 Tests
1. **Module Download**: Test S3 integration and caching
2. **Multi-module**: Run multiple WASM modules simultaneously
3. **Error Handling**: Test malformed requests and WASM errors

### Phase 3 Tests
1. **Performance**: Benchmark cold start times vs traditional MCP
2. **Security**: Test WASM sandboxing and capability isolation
3. **End-to-end**: Full workflow from frontend submission to desktop execution

## Next Steps After Proof of Concept

1. **Expand WASM Interface**: Add comprehensive MCP protocol mapping
2. **Build Backend**: Implement module registry and S3 integration
3. **Create Templates**: Rust/TypeScript/Python starter templates
4. **Develop Frontend**: Module marketplace interface
5. **CLI Tool**: Development workflow automation
