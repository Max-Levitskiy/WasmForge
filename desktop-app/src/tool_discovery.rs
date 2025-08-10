use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::config::{ModuleConfig, ToolConfig};
use crate::wasm_executor::{WasmExecutor, FuncSignature};

#[derive(Debug, Clone)]
pub struct DiscoveredTool {
    pub name: String,
    pub module_name: String,
    pub function_name: String,
    pub description: String,
    pub schema: Value,
    pub signature: FuncSignature,
    pub pattern: String,
}

pub struct ToolDiscovery {
    discovered_tools: HashMap<String, DiscoveredTool>,
    tool_configs: HashMap<String, HashMap<String, ToolConfig>>, // module_name -> tool_name -> config
}

impl ToolDiscovery {
    pub fn new() -> Self {
        Self {
            discovered_tools: HashMap::new(),
            tool_configs: HashMap::new(),
        }
    }

    pub fn load_tool_configs(&mut self, modules: &[ModuleConfig]) {
        for module in modules {
            if let Some(tools) = &module.tools {
                let mut module_tools = HashMap::new();
                for tool in tools {
                    module_tools.insert(tool.name.clone(), tool.clone());
                }
                self.tool_configs.insert(module.name.clone(), module_tools);
            }
        }
    }

    pub fn discover_tools_from_executor(&mut self, executor: &WasmExecutor) -> Result<usize> {
        let mut discovered_count = 0;
        
        let all_functions = executor.get_all_functions();
        
        for (module_name, functions) in all_functions {
            // Check if this module has both validate_url and process_response functions
            let has_validate_url = functions.contains(&"validate_url".to_string());
            let has_process_response = functions.contains(&"process_response".to_string());
            
            // Create special "fetch" tool if both functions are present
            if has_validate_url && has_process_response {
                let fetch_tool = DiscoveredTool {
                    name: "fetch".to_string(),
                    module_name: module_name.clone(),
                    function_name: "fetch".to_string(), // Virtual function name
                    description: format!("Fetch content from a URL using WASM validation and processing (from module: {})", module_name),
                    schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "url": {"type": "string", "description": "The URL to fetch"}
                        },
                        "required": ["url"]
                    }),
                    signature: FuncSignature {
                        params: vec![wasmtime::ValType::I32, wasmtime::ValType::I32],
                        results: vec![wasmtime::ValType::I32],
                    },
                    pattern: "ptr_len_to_i32".to_string(),
                };
                
                self.discovered_tools.insert("fetch".to_string(), fetch_tool);
                discovered_count += 1;
            }
            
            for function_name in functions {
                if let Ok(signature) = executor.get_function_signature(&module_name, &function_name) {
                    if let Some(tool) = self.analyze_function(&module_name, &function_name, signature)? {
                        let tool_key = format!("{}::{}", module_name, function_name);
                        self.discovered_tools.insert(tool_key, tool);
                        discovered_count += 1;
                    }
                }
            }
        }

        println!("✓ Discovered {} tools from WASM modules", discovered_count);
        Ok(discovered_count)
    }

    fn analyze_function(
        &self,
        module_name: &str,
        function_name: &str,
        signature: FuncSignature,
    ) -> Result<Option<DiscoveredTool>> {
        // Skip internal or system functions
        if function_name.starts_with('_') || function_name.starts_with("__") {
            return Ok(None);
        }

        // Determine the function pattern and create appropriate schema
        // First check if this is a pointer/length function by name
        let (pattern, schema, description) = if matches!(function_name, "validate_url" | "process_response" | "prepare_http_get" | "prepare_file_read" | "prepare_file_write" | "prepare_shell_exec" | "prepare_recommend_mcps") && signature.matches_pattern("ptr_len_to_i32") {
            // Handle different ptr_len_to_i32 function types with appropriate schemas
            let (schema, description) = match function_name {
                "prepare_http_get" => (
                    json!({
                        "type": "object",
                        "properties": {
                            "url": {"type": "string", "description": "The URL to fetch via HTTP GET request"}
                        },
                        "required": ["url"]
                    }),
                    self.generate_description(module_name, function_name, "Fetch content from a URL using async HTTP GET with WASM validation")
                ),
                "prepare_file_read" => (
                    json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "The file path to read"}
                        },
                        "required": ["path"]
                    }),
                    self.generate_description(module_name, function_name, "Read file content with WASM path validation")
                ),
                "prepare_file_write" => (
                    json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "The file path to write to"},
                            "content": {"type": "string", "description": "The content to write to the file"}
                        },
                        "required": ["path", "content"]
                    }),
                    self.generate_description(module_name, function_name, "Write content to file with WASM path validation")
                ),
                "prepare_shell_exec" => (
                    json!({
                        "type": "object",
                        "properties": {
                            "command": {"type": "string", "description": "The shell command to execute (validated by WASM and host)"}
                        },
                        "required": ["command"]
                    }),
                    self.generate_description(module_name, function_name, "Execute a simple shell command with WASM validation")
                ),
                "prepare_recommend_mcps" => (
                    json!({
                        "type": "object",
                        "properties": {
                            "task": {"type": "string", "description": "Describe your task and we'll recommend suitable tools"}
                        },
                        "required": ["task"]
                    }),
                    self.generate_description(module_name, function_name, "Recommend relevant MCP tools based on a task description")
                ),
                _ => (
                    json!({
                        "type": "object",
                        "properties": {
                            "data": {"type": "string", "description": "Data to process"}
                        },
                        "required": ["data"]
                    }),
                    self.generate_description(module_name, function_name, "Processes string data and returns an integer status")
                )
            };
            
            (
                "ptr_len_to_i32".to_string(),
                schema,
                description
            )
        } else if signature.matches_pattern("i32_i32_to_i32") {
            (
                "i32_i32_to_i32".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "First integer parameter"},
                        "b": {"type": "number", "description": "Second integer parameter"}
                    },
                    "required": ["a", "b"]
                }),
                self.generate_description(module_name, function_name, "Takes two integers and returns an integer")
            )
        } else if signature.matches_pattern("no_params_to_i32") {
            (
                "no_params_to_i32".to_string(),
                json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
                self.generate_description(module_name, function_name, "Takes no parameters and returns an integer")
            )
        } else {
            // Unsupported signature pattern
            return Ok(None);
        };

        // Check if we have custom configuration for this tool
        let (final_description, final_schema) = if let Some(module_tools) = self.tool_configs.get(module_name) {
            if let Some(tool_config) = module_tools.get(function_name) {
                (
                    tool_config.description.clone().unwrap_or(description),
                    tool_config.parameters.clone().unwrap_or(schema),
                )
            } else {
                (description, schema)
            }
        } else {
            (description, schema)
        };

        // Create namespaced tool name
        let tool_name = if module_name == "test-module" {
            // For backwards compatibility with test module
            function_name.to_string()
        } else {
            format!("{}_{}", module_name.replace('-', "_"), function_name)
        };

        Ok(Some(DiscoveredTool {
            name: tool_name,
            module_name: module_name.to_string(),
            function_name: function_name.to_string(),
            description: final_description,
            schema: final_schema,
            signature,
            pattern,
        }))
    }

    fn generate_description(&self, module_name: &str, function_name: &str, default: &str) -> String {
        // Try to generate a more meaningful description based on function name
        let description = match function_name {
            "add" => "Add two numbers using WebAssembly",
            "subtract" | "sub" => "Subtract two numbers using WebAssembly",
            "multiply" | "mul" => "Multiply two numbers using WebAssembly",
            "divide" | "div" => "Divide two numbers using WebAssembly",
            "validate_url" => "Validate URL format using WebAssembly",
            "process_response" => "Process HTTP response using WebAssembly",
            "prepare_http_get" => "Fetch content from a URL using async HTTP GET with WASM validation",
            "prepare_file_read" => "Read file content with WASM path validation",
            "prepare_file_write" => "Write content to file with WASM path validation",
            "prepare_shell_exec" => "Execute a simple shell command with WASM validation",
            "prepare_recommend_mcps" => "Recommend relevant MCP tools based on a task description",
            "hash" | "sha256" => "Calculate hash of input data",
            "encrypt" => "Encrypt data using WebAssembly",
            "decrypt" => "Decrypt data using WebAssembly",
            "compress" => "Compress data using WebAssembly",
            "decompress" => "Decompress data using WebAssembly",
            name if name.contains("validate") => "Validate input data using WebAssembly",
            name if name.contains("process") => "Process input data using WebAssembly",
            name if name.contains("parse") => "Parse input data using WebAssembly",
            name if name.contains("format") => "Format input data using WebAssembly",
            _ => default,
        };

        format!("{} (from module: {})", description, module_name)
    }

    pub fn get_all_tools(&self) -> &HashMap<String, DiscoveredTool> {
        &self.discovered_tools
    }

    pub fn get_tool(&self, tool_name: &str) -> Option<&DiscoveredTool> {
        self.discovered_tools.get(tool_name)
    }

    pub fn find_tool_by_name(&self, name: &str) -> Option<&DiscoveredTool> {
        // First try exact match
        if let Some(tool) = self.discovered_tools.get(name) {
            return Some(tool);
        }

        // Then try to find by tool name (without module prefix)
        for tool in self.discovered_tools.values() {
            if tool.name == name {
                return Some(tool);
            }
        }

        None
    }

    pub fn get_mcp_tools_schema(&self) -> Value {
        let mut tools = Vec::new();

        for tool in self.discovered_tools.values() {
            tools.push(json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.schema
            }));
        }

        json!({ "tools": tools })
    }

    pub fn get_tools_by_module(&self, module_name: &str) -> Vec<&DiscoveredTool> {
        self.discovered_tools
            .values()
            .filter(|tool| tool.module_name == module_name)
            .collect()
    }

    pub fn get_tool_count(&self) -> usize {
        self.discovered_tools.len()
    }

    pub fn clear(&mut self) {
        self.discovered_tools.clear();
    }

    pub fn print_discovered_tools(&self) {
        if self.discovered_tools.is_empty() {
            println!("No tools discovered");
            return;
        }

        println!("\n📋 Discovered Tools:");
        println!("┌─────────────────────────────────────────────────────────────────┐");
        
        let mut tools_by_module: HashMap<String, Vec<&DiscoveredTool>> = HashMap::new();
        for tool in self.discovered_tools.values() {
            tools_by_module
                .entry(tool.module_name.clone())
                .or_insert_with(Vec::new)
                .push(tool);
        }

        for (module_name, tools) in tools_by_module {
            println!("│ 📦 Module: {:<52} │", module_name);
            println!("├─────────────────────────────────────────────────────────────────┤");
            
            for tool in tools {
                println!("│ 🔧 {:<15} {} │", tool.name, tool.description);
                println!("│    └─ Pattern: {:<47} │", tool.pattern);
            }
            println!("├─────────────────────────────────────────────────────────────────┤");
        }
        println!("└─────────────────────────────────────────────────────────────────┘");
        println!("Total: {} tools", self.discovered_tools.len());
    }
}

