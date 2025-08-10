use wasmtime::*;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader};
use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};

mod config;
mod module_manager;
mod wasm_executor;
mod tool_discovery;

use config::Config;
use module_manager::ModuleManager;
use wasm_executor::WasmExecutor;
use tool_discovery::ToolDiscovery;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MCPRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MCPResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

struct WasmForgeServer {
    executor: WasmExecutor,
    tool_discovery: ToolDiscovery,
    module_manager: ModuleManager,
    config: Config,
}

impl WasmForgeServer {
    async fn new(config: Config) -> Result<Self, anyhow::Error> {
        let mut module_manager = ModuleManager::new(config.clone())?;
        let mut executor = WasmExecutor::new()?;
        let mut tool_discovery = ToolDiscovery::new();

        // Load tool configurations
        tool_discovery.load_tool_configs(&config.modules);

        // Load all modules
        module_manager.load_all_modules().await?;
        executor.load_modules_from_manager(&module_manager).await?;
        
        // Discover tools from loaded modules
        tool_discovery.discover_tools_from_executor(&executor)?;
        tool_discovery.print_discovered_tools();

        Ok(Self {
            executor,
            tool_discovery,
            module_manager,
            config,
        })
    }
}

async fn handle_mcp_message(request: MCPRequest, server: &mut WasmForgeServer) -> MCPResponse {
    match request.method.as_str() {
        "initialize" => MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "wasmforge",
                    "version": "0.1.0"
                }
            })),
            error: None,
        },
        "tools/list" => MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(server.tool_discovery.get_mcp_tools_schema()),
            error: None,
        },
        "tools/call" => {
            match handle_tool_call(request.clone(), server).await {
                Ok(response) => response,
                Err(e) => MCPResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(serde_json::json!({
                        "code": -32000,
                        "message": e.to_string()
                    })),
                }
            }
        },
        _ => MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(serde_json::json!({
                "code": -32601,
                "message": "Method not found"
            })),
        },
    }
}

async fn handle_tool_call(request: MCPRequest, server: &mut WasmForgeServer) -> Result<MCPResponse, anyhow::Error> {
    let params = request.params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
    let tool_name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;
    let arguments = params.get("arguments")
        .ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Find the tool in our discovery system
    let tool = server.tool_discovery.find_tool_by_name(tool_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown tool: {}", tool_name))?;

    // Call the appropriate function based on the tool's pattern
    let result_text = match tool.pattern.as_str() {
        "i32_i32_to_i32" => {
            let a = arguments.get("a")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing or invalid parameter 'a'"))? as i32;
            let b = arguments.get("b")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing or invalid parameter 'b'"))? as i32;

            let result = server.executor.call_function_i32_i32_to_i32(
                &tool.module_name,
                &tool.function_name,
                a,
                b,
            )?;

            format!("WASM calculation result: {} (from {}::{})", result, tool.module_name, tool.function_name)
        }
        "ptr_len_to_i32" => {
            // Handle different async operation patterns
            if tool.function_name == "validate_url" {
                let url = arguments.get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing or invalid parameter 'url'"))?;

                let result = server.executor.call_function_ptr_len_to_i32(
                    &tool.module_name,
                    &tool.function_name,
                    url.as_bytes(),
                )?;

                format!("URL validation result: {} (1=valid, 0=invalid)", result)
            } else if tool.function_name == "prepare_http_get" {
                // New HTTP GET async operation
                let url = arguments.get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing URL parameter"))?;

                let content = server.executor.http_get_with_validation(&tool.module_name, url).await?;
                
                format!("HTTP GET successful!\nURL: {}\nContent length: {} bytes\n\nContent preview (first 500 chars):\n{}", 
                    url,
                    content.len(),
                    if content.len() > 500 { 
                        format!("{}...", &content[..500]) 
                    } else { 
                        content 
                    }
                )
            } else if tool.function_name == "prepare_file_read" {
                // New file reading async operation
                let file_path = arguments.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;

                let content = server.executor.read_file_with_validation(&tool.module_name, file_path).await?;
                
                format!("File read successful!\nPath: {}\nContent length: {} bytes\n\nContent:\n{}", 
                    file_path,
                    content.len(),
                    content
                )
            } else if tool.function_name == "prepare_file_write" {
                // New file writing async operation
                let file_path = arguments.get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;
                
                let content = arguments.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing content parameter"))?;

                let result = server.executor.write_file_with_validation(&tool.module_name, file_path, content).await?;
                
                format!("File write successful!\nPath: {}\nContent length: {} bytes\nResult: {}", 
                    file_path,
                    content.len(),
                    result
                )
            } else if tool.function_name == "prepare_shell_exec" {
                // Shell execution with dual validation and config-driven allow-list
                let cmd = arguments.get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing command parameter"))?;

                // Resolve allowed commands: tool security -> module metadata -> defaults
                let module_config_opt = server.config.find_module(&tool.module_name);
                let mut allowed: Vec<String> = vec![
                    "echo".to_string(),
                    "cat".to_string(),
                    "ls".to_string(),
                    "wc".to_string(),
                    "uname".to_string(),
                ];

                if let Some(module_config) = module_config_opt {
                    // Prefer structured tool security config
                    if let Some(tools) = &module_config.tools {
                        if let Some(tool_cfg) = tools.iter().find(|t| t.function_name == "prepare_shell_exec") {
                            if let Some(sec) = &tool_cfg.security {
                                if let Some(list) = &sec.allowed_commands {
                                    if !list.is_empty() { allowed = list.clone(); }
                                }
                            }
                        }
                    }

                    // Fallback: metadata CSV
                    if let Some(meta) = &module_config.metadata {
                        if let Some(csv) = meta.get("allowed_commands_csv") {
                            let parsed: Vec<String> = csv.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                            if !parsed.is_empty() { allowed = parsed; }
                        }
                    }
                }

                let result = server.executor.execute_shell_with_validation(&tool.module_name, cmd, &allowed).await?;
                result
            } else if tool.name == "fetch" {
                // Legacy fetch tool for backward compatibility
                let url = arguments.get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing URL parameter"))?;

                let content = server.executor.fetch_url_with_validation(&tool.module_name, url).await?;
                
                format!("URL: {}\n\nContent (first 500 chars):\n{}", 
                    url, 
                    if content.len() > 500 { 
                        format!("{}...", &content[..500]) 
                    } else { 
                        content 
                    }
                )
            } else if tool.function_name == "prepare_recommend_mcps" {
                let task = arguments.get("task")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing task parameter"))?;

                // Validate via WASM first
                let ok = server.executor.call_function_ptr_len_to_i32(
                    &tool.module_name,
                    &tool.function_name,
                    task.as_bytes(),
                )? == 1;
                if !ok { return Err(anyhow::anyhow!("Task rejected by WASM validation")); }

                // Build recommendations from discovered tools
                let query = task.to_lowercase();
                let tools = server.tool_discovery.get_all_tools();

                let mut categories: Vec<serde_json::Value> = Vec::new();

                // Helpers to collect methods by function name
                let has_fn = |fname: &str| tools.values().any(|t| t.function_name == fname);
                let collect_methods = |fnames: &[&str]| -> Vec<serde_json::Value> {
                    let mut v = Vec::new();
                    for t in tools.values() {
                        if fnames.contains(&t.function_name.as_str()) {
                            v.push(serde_json::json!({
                                "name": t.name,
                                "inputSchema": t.schema
                            }));
                        }
                    }
                    v
                };

                // Scoring keywords
                let contains_any = |words: &[&str]| words.iter().any(|w| query.contains(w));

                // Web browser
                if has_fn("prepare_http_get") && contains_any(&["download","fetch","http","https","url","get","retrieve","request"]) {
                    categories.push(serde_json::json!({
                        "name": "web_browser",
                        "description": "Fetch content via HTTP GET with WASM validation",
                        "methods": collect_methods(&["prepare_http_get"]) 
                    }));
                }

                // File ops
                if (has_fn("prepare_file_read") || has_fn("prepare_file_write")) && contains_any(&["save","file","write","read","open","load","store"]) {
                    categories.push(serde_json::json!({
                        "name": "file_ops",
                        "description": "Read and write files with WASM path validation",
                        "methods": collect_methods(&["prepare_file_read","prepare_file_write"]) 
                    }));
                }

                // Shell executor
                if has_fn("prepare_shell_exec") && contains_any(&["shell","bash","command","execute","run","ls","echo","cat","wc","uname","terminal"]) {
                    categories.push(serde_json::json!({
                        "name": "shell_executor",
                        "description": "Execute simple whitelisted shell commands with WASM validation",
                        "methods": collect_methods(&["prepare_shell_exec"]) 
                    }));
                }

                // Fallback: if none matched, include the three families if present
                if categories.is_empty() {
                    if has_fn("prepare_http_get") { categories.push(serde_json::json!({
                        "name": "web_browser",
                        "description": "Fetch content via HTTP GET with WASM validation",
                        "methods": collect_methods(&["prepare_http_get"]) 
                    })); }
                    if has_fn("prepare_file_read") || has_fn("prepare_file_write") { categories.push(serde_json::json!({
                        "name": "file_ops",
                        "description": "Read and write files with WASM path validation",
                        "methods": collect_methods(&["prepare_file_read","prepare_file_write"]) 
                    })); }
                    if has_fn("prepare_shell_exec") { categories.push(serde_json::json!({
                        "name": "shell_executor",
                        "description": "Execute simple whitelisted shell commands with WASM validation",
                        "methods": collect_methods(&["prepare_shell_exec"]) 
                    })); }
                }

                let json_text = serde_json::to_string_pretty(&serde_json::json!(categories))?;
                json_text
            } else {
                let data = arguments.get("data")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing or invalid parameter 'data'"))?;

                let result = server.executor.call_function_ptr_len_to_i32(
                    &tool.module_name,
                    &tool.function_name,
                    data.as_bytes(),
                )?;

                format!("WASM processing result: {} (from {}::{})", result, tool.module_name, tool.function_name)
            }
        }
        "no_params_to_i32" => {
            let result = server.executor.call_function_no_params_to_i32(
                &tool.module_name,
                &tool.function_name,
            )?;

            format!("WASM result: {} (from {}::{})", result, tool.module_name, tool.function_name)
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported function pattern: {}", tool.pattern));
        }
    };

    Ok(MCPResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": result_text
                }
            ]
        })),
        error: None,
    })
}

#[derive(Parser, Debug)]
#[command(name = "wasmforge-desktop")]
#[command(about = "WasmForge MCP Server")]
struct Args {
    /// Port to listen on for TCP connections (if not specified, uses stdio)
    #[arg(short, long)]
    port: Option<u16>,
    
    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

async fn handle_tcp_connection(mut stream: TcpStream, server: std::sync::Arc<tokio::sync::Mutex<WasmForgeServer>>) -> Result<(), anyhow::Error> {
    let (reader, mut writer) = stream.split();
    let mut reader = AsyncBufReader::new(reader);
    let mut line = String::new();
    
    loop {
        line.clear();
        match reader.read_line(&mut line).await? {
            0 => break, // Connection closed
            _ => {
                if line.trim().is_empty() {
                    continue;
                }
                
                match serde_json::from_str::<MCPRequest>(&line) {
                    Ok(request) => {
                        let mut server = server.lock().await;
                        let response = handle_mcp_message(request, &mut server).await;
                        let response_json = serde_json::to_string(&response)?;
                        writer.write_all(response_json.as_bytes()).await?;
                        writer.write_all(b"\n").await?;
                        writer.flush().await?;
                    }
                    Err(e) => {
                        eprintln!("Failed to parse request: {}", e);
                    }
                }
            }
        }
    }
    
    Ok(())
}

async fn run_tcp_server(host: &str, port: u16, server: WasmForgeServer) -> Result<(), anyhow::Error> {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await?;
    let server = std::sync::Arc::new(tokio::sync::Mutex::new(server));
    
    eprintln!("WasmForge MCP Server listening on {}", addr);
    
    loop {
        let (stream, addr) = listener.accept().await?;
        eprintln!("New connection from {}", addr);
        
        let server_clone = server.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_tcp_connection(stream, server_clone).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}

async fn run_stdio_server(mut server: WasmForgeServer) -> Result<(), anyhow::Error> {
    eprintln!("WasmForge MCP Server started on stdio");
    
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        match serde_json::from_str::<MCPRequest>(&line) {
            Ok(request) => {
                let response = handle_mcp_message(request, &mut server).await;
                println!("{}", serde_json::to_string(&response)?);
            }
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    
    // Load configuration
    let config_path = Config::get_config_path();
    let config = Config::load_or_create_default(&config_path)?;
    
    // Validate configuration
    config.validate()?;
    
    println!("📋 WasmForge MCP Server starting...");
    println!("Config: {}", config_path.display());
    
    // Initialize the server with all modules
    let server = WasmForgeServer::new(config).await?;
    
    match args.port {
        Some(port) => {
            run_tcp_server(&args.host, port, server).await
        }
        None => {
            run_stdio_server(server).await
        }
    }
}
