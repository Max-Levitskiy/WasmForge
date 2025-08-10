use anyhow::{Context, Result};
use std::collections::HashMap;
use wasmtime::*;

use crate::module_manager::{ModuleManager, ModuleMetadata};

pub struct LoadedModule {
    pub metadata: ModuleMetadata,
    pub module: Module,
    pub instance: Instance,
    pub store: Store<()>,
}

pub struct WasmExecutor {
    engine: Engine,
    modules: HashMap<String, LoadedModule>,
}

impl WasmExecutor {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        Ok(Self {
            engine,
            modules: HashMap::new(),
        })
    }

    pub async fn load_modules_from_manager(&mut self, module_manager: &ModuleManager) -> Result<()> {
        for (name, metadata) in module_manager.get_loaded_modules() {
            match self.load_module_from_metadata(module_manager, metadata).await {
                Ok(_) => {
                    println!("✓ WASM executor loaded module: {}", name);
                }
                Err(e) => {
                    eprintln!("✗ Failed to load module '{}' into executor: {}", name, e);
                }
            }
        }
        Ok(())
    }

    async fn load_module_from_metadata(
        &mut self,
        module_manager: &ModuleManager,
        metadata: &ModuleMetadata,
    ) -> Result<()> {
        let wasm_bytes = module_manager.get_module_bytes(&metadata.name)?;
        
        let module = Module::new(&self.engine, &wasm_bytes)
            .with_context(|| format!("Failed to compile WASM module: {}", metadata.name))?;

        let mut store = Store::new(&self.engine, ());
        let instance = Instance::new(&mut store, &module, &[])
            .with_context(|| format!("Failed to instantiate WASM module: {}", metadata.name))?;

        let loaded_module = LoadedModule {
            metadata: metadata.clone(),
            module,
            instance,
            store,
        };

        self.modules.insert(metadata.name.clone(), loaded_module);
        Ok(())
    }

    pub fn get_module_functions(&self, module_name: &str) -> Result<Vec<String>> {
        let module = self.modules.get(module_name)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not loaded", module_name))?;

        let mut functions = Vec::new();
        
        // Get all exports from the module
        for export in module.module.exports() {
            if matches!(export.ty(), ExternType::Func(_)) {
                functions.push(export.name().to_string());
            }
        }

        Ok(functions)
    }

    pub fn get_all_functions(&self) -> HashMap<String, Vec<String>> {
        let mut all_functions = HashMap::new();
        
        for (module_name, _) in &self.modules {
            if let Ok(functions) = self.get_module_functions(module_name) {
                all_functions.insert(module_name.clone(), functions);
            }
        }
        
        all_functions
    }

    pub fn call_function_i32_i32_to_i32(
        &mut self,
        module_name: &str,
        function_name: &str,
        a: i32,
        b: i32,
    ) -> Result<i32> {
        let module = self.modules.get_mut(module_name)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not loaded", module_name))?;

        let func = module.instance
            .get_typed_func::<(i32, i32), i32>(&mut module.store, function_name)
            .with_context(|| format!("Function '{}' not found in module '{}'", function_name, module_name))?;

        let result = func.call(&mut module.store, (a, b))
            .with_context(|| format!("Failed to call function '{}' in module '{}'", function_name, module_name))?;

        Ok(result)
    }

    pub fn call_function_ptr_len_to_i32(
        &mut self,
        module_name: &str,
        function_name: &str,
        data: &[u8],
    ) -> Result<i32> {
        let module = self.modules.get_mut(module_name)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not loaded", module_name))?;

        // Get memory export
        let memory = module.instance
            .get_memory(&mut module.store, "memory")
            .ok_or_else(|| anyhow::anyhow!("No memory export found in module '{}'", module_name))?;

        // Allocate space in WASM memory for the data
        let data_ptr = 1024; // Simple fixed offset - in production, use proper allocation
        memory.write(&mut module.store, data_ptr, data)
            .context("Failed to write data to WASM memory")?;

        let func = module.instance
            .get_typed_func::<(i32, i32), i32>(&mut module.store, function_name)
            .with_context(|| format!("Function '{}' not found in module '{}'", function_name, module_name))?;

        let result = func.call(&mut module.store, (data_ptr as i32, data.len() as i32))
            .with_context(|| format!("Failed to call function '{}' in module '{}'", function_name, module_name))?;

        Ok(result)
    }

    pub fn call_function_no_params_to_i32(
        &mut self,
        module_name: &str,
        function_name: &str,
    ) -> Result<i32> {
        let module = self.modules.get_mut(module_name)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not loaded", module_name))?;

        let func = module.instance
            .get_typed_func::<(), i32>(&mut module.store, function_name)
            .with_context(|| format!("Function '{}' not found in module '{}'", function_name, module_name))?;

        let result = func.call(&mut module.store, ())
            .with_context(|| format!("Failed to call function '{}' in module '{}'", function_name, module_name))?;

        Ok(result)
    }

    pub fn get_function_signature(&self, module_name: &str, function_name: &str) -> Result<FuncSignature> {
        let module = self.modules.get(module_name)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not loaded", module_name))?;

        // Find the function export
        for export in module.module.exports() {
            if export.name() == function_name {
                if let ExternType::Func(func_type) = export.ty() {
                    return Ok(FuncSignature {
                        params: func_type.params().collect(),
                        results: func_type.results().collect(),
                    });
                }
            }
        }

        Err(anyhow::anyhow!("Function '{}' not found in module '{}'", function_name, module_name))
    }

    pub async fn fetch_url_with_validation(
        &mut self,
        module_name: &str,
        url: &str,
    ) -> Result<String> {
        // First validate URL using WASM
        let is_valid = self.call_function_ptr_len_to_i32(
            module_name,
            "validate_url",
            url.as_bytes(),
        )? == 1;

        if !is_valid {
            return Err(anyhow::anyhow!("Invalid URL according to WASM validation"));
        }

        // Make HTTP request
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        let text = response.text().await?;

        // Process response using WASM
        let status = self.call_function_ptr_len_to_i32(
            module_name,
            "process_response",
            text.as_bytes(),
        )?;

        if status == 200 {
            Ok(text)
        } else {
            Err(anyhow::anyhow!("WASM processing failed with status: {}", status))
        }
    }

    // New async operation: HTTP GET with WASM validation
    pub async fn http_get_with_validation(
        &mut self,
        module_name: &str,
        url: &str,
    ) -> Result<String> {
        // First, let WASM validate/prepare the request
        let is_valid = self.call_function_ptr_len_to_i32(
            module_name,
            "prepare_http_get",
            url.as_bytes(),
        )? == 1;

        if !is_valid {
            return Err(anyhow::anyhow!("URL rejected by WASM validation: {}", url));
        }

        // Execute the HTTP request asynchronously on the host
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("WasmForge/0.1.0")
            .build()?;

        let response = client.get(url).send().await
            .with_context(|| format!("Failed to fetch URL: {}", url))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP request failed with status: {}", response.status()));
        }

        let text = response.text().await
            .context("Failed to read response body as text")?;

        Ok(text)
    }

    // New async operation: File reading with WASM validation
    pub async fn read_file_with_validation(
        &mut self,
        module_name: &str,
        file_path: &str,
    ) -> Result<String> {
        // First, let WASM validate/prepare the file path
        let is_safe = self.call_function_ptr_len_to_i32(
            module_name,
            "prepare_file_read",
            file_path.as_bytes(),
        )? == 1;

        if !is_safe {
            return Err(anyhow::anyhow!("File path rejected by WASM validation: {}", file_path));
        }

        // Execute the file read asynchronously on the host
        let content = tokio::fs::read_to_string(file_path).await
            .with_context(|| format!("Failed to read file: {}", file_path))?;

        // Optional: validate content size
        if content.len() > 1024 * 1024 {  // 1MB limit
            return Err(anyhow::anyhow!("File too large: {} bytes", content.len()));
        }

        Ok(content)
    }

    // New async operation: File writing with WASM validation
    pub async fn write_file_with_validation(
        &mut self,
        module_name: &str,
        file_path: &str,
        content: &str,
    ) -> Result<String> {
        // First, let WASM validate/prepare the file path
        let is_safe = self.call_function_ptr_len_to_i32(
            module_name,
            "prepare_file_write",
            file_path.as_bytes(),
        )? == 1;

        if !is_safe {
            return Err(anyhow::anyhow!("File path rejected by WASM validation for writing: {}", file_path));
        }

        // Additional safety check: content size limit
        if content.len() > 10 * 1024 * 1024 {  // 10MB limit
            return Err(anyhow::anyhow!("Content too large: {} bytes (max 10MB)", content.len()));
        }

        // Execute the file write asynchronously on the host
        tokio::fs::write(file_path, content).await
            .with_context(|| format!("Failed to write file: {}", file_path))?;

        Ok(format!("Successfully wrote {} bytes to {}", content.len(), file_path))
    }

    pub fn get_loaded_modules(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }

    pub fn is_module_loaded(&self, module_name: &str) -> bool {
        self.modules.contains_key(module_name)
    }

    pub fn unload_module(&mut self, module_name: &str) -> Result<()> {
        if self.modules.remove(module_name).is_some() {
            println!("Unloaded module: {}", module_name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Module '{}' not loaded", module_name))
        }
    }

    pub async fn reload_module(&mut self, module_manager: &ModuleManager, module_name: &str) -> Result<()> {
        // Unload existing module if present
        let _ = self.unload_module(module_name);

        // Load fresh module
        if let Some(metadata) = module_manager.get_module_metadata(module_name) {
            self.load_module_from_metadata(module_manager, metadata).await?;
            println!("✓ Reloaded module: {}", module_name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Module '{}' not found in module manager", module_name))
        }
    }

    // Execute a shell command validated by WASM and restricted by an allow-list
    pub async fn execute_shell_with_validation(
        &mut self,
        module_name: &str,
        command: &str,
        allowed_commands: &[String],
    ) -> Result<String> {
        // First, let WASM validate the raw command text
        let is_valid = self.call_function_ptr_len_to_i32(
            module_name,
            "prepare_shell_exec",
            command.as_bytes(),
        )? == 1;

        if !is_valid {
            return Err(anyhow::anyhow!("Command rejected by WASM validation"));
        }

        // Server-side enforcement: allow-list and timeout
        let tokens: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
        if tokens.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }

        let program = &tokens[0];
        let is_allowed = allowed_commands.iter().any(|c| c == program);
        if !is_allowed {
            return Err(anyhow::anyhow!(format!("Command '{}' is not allowed", program)));
        }

        let args: Vec<&str> = tokens.iter().skip(1).map(|s| s.as_str()).collect();

        use tokio::process::Command as TokioCommand;
        use tokio::time::{timeout, Duration};

        let mut child = TokioCommand::new(program)
            .args(&args)
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to spawn command: {}", program))?;

        let output = match timeout(Duration::from_secs(10), child.wait_with_output()).await {
            Ok(res) => res.with_context(|| "Failed to wait for command output")?,
            Err(_) => return Err(anyhow::anyhow!("Command timed out after 10s")),
        };

        let exit_code = output.status.code().unwrap_or(-1);
        let mut stdout_text = String::from_utf8_lossy(&output.stdout).to_string();
        let mut stderr_text = String::from_utf8_lossy(&output.stderr).to_string();

        // Truncate large outputs
        let max_len = 4096;
        if stdout_text.len() > max_len { stdout_text.truncate(max_len); }
        if stderr_text.len() > max_len { stderr_text.truncate(max_len); }

        Ok(format!(
            "Shell execution completed.\nExit code: {}\n\nSTDOUT (truncated):\n{}\n\nSTDERR (truncated):\n{}",
            exit_code, stdout_text, stderr_text
        ))
    }
}

#[derive(Debug, Clone)]
pub struct FuncSignature {
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

impl FuncSignature {
    pub fn param_count(&self) -> usize {
        self.params.len()
    }

    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    pub fn matches_pattern(&self, pattern: &str) -> bool {
        match pattern {
            "i32_i32_to_i32" => {
                self.params.len() == 2 
                    && matches!(self.params[0], ValType::I32)
                    && matches!(self.params[1], ValType::I32)
                    && self.results.len() == 1
                    && matches!(self.results[0], ValType::I32)
            }
            "ptr_len_to_i32" => {
                self.params.len() == 2
                    && matches!(self.params[0], ValType::I32)  // pointer
                    && matches!(self.params[1], ValType::I32)  // length
                    && self.results.len() == 1
                    && matches!(self.results[0], ValType::I32)
            }
            "no_params_to_i32" => {
                self.params.is_empty()
                    && self.results.len() == 1
                    && matches!(self.results[0], ValType::I32)
            }
            _ => false,
        }
    }
}