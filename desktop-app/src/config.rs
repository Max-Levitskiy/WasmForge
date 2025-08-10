use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub modules: Vec<ModuleConfig>,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub default_port: Option<u16>,
    pub default_host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub source: ModuleSource,
    pub enabled: bool,
    pub tools: Option<Vec<ToolConfig>>,
    pub metadata: Option<HashMap<String, String>>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub name: String,
    pub description: Option<String>,
    pub function_name: String,
    pub parameters: Option<serde_json::Value>,
    pub security: Option<ToolSecurityConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSecurityConfig {
    pub allowed_commands: Option<Vec<String>>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ModuleSource {
    Local { path: PathBuf },
    Http { url: String, checksum: Option<String> },
    Registry { name: String, version: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub directory: PathBuf,
    pub max_size_mb: u64,
    pub ttl_hours: u64,
}

impl Default for Config {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wasmforge")
            .join("modules");

        Config {
            server: ServerConfig {
                name: "wasmforge".to_string(),
                version: "0.1.0".to_string(),
                default_port: None,
                default_host: "127.0.0.1".to_string(),
            },
            modules: vec![
                ModuleConfig {
                    name: "test-module".to_string(),
                    version: Some("0.1.0".to_string()),
                    description: Some("Test WebAssembly module with basic functions".to_string()),
                    source: ModuleSource::Local {
                        path: PathBuf::from("test-modules/test_module.wasm"),
                    },
                    enabled: true,
                    tools: Some(vec![
                        ToolConfig {
                            name: "add".to_string(),
                            description: Some("Add two numbers".to_string()),
                            function_name: "add".to_string(),
                            parameters: Some(serde_json::json!({
                                "type": "object",
                                "properties": {
                                    "a": {"type": "number"},
                                    "b": {"type": "number"}
                                },
                                "required": ["a", "b"]
                            })),
                            security: None,
                        },
                        ToolConfig {
                            name: "validate_url".to_string(),
                            description: Some("Validate URL format".to_string()),
                            function_name: "validate_url".to_string(),
                            parameters: Some(serde_json::json!({
                                "type": "object",
                                "properties": {
                                    "url": {"type": "string"}
                                },
                                "required": ["url"]
                            })),
                            security: None,
                        },
                    ]),
                    metadata: None,
                }
            ],
            cache: CacheConfig {
                directory: cache_dir,
                max_size_mb: 100,
                ttl_hours: 24,
            },
        }
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;
        
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.as_ref().display()))?;
        
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;
        
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        fs::write(&path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;
        
        Ok(())
    }

    pub fn load_or_create_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            Self::load_from_file(path)
        } else {
            let config = Self::default();
            config.save_to_file(&path)?;
            Ok(config)
        }
    }

    pub fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wasmforge")
            .join("config.toml")
    }

    pub fn enabled_modules(&self) -> impl Iterator<Item = &ModuleConfig> {
        self.modules.iter().filter(|m| m.enabled)
    }

    pub fn find_module(&self, name: &str) -> Option<&ModuleConfig> {
        self.modules.iter().find(|m| m.name == name)
    }

    pub fn validate(&self) -> Result<()> {
        // Validate that cache directory can be created
        fs::create_dir_all(&self.cache.directory)
            .with_context(|| format!("Cannot create cache directory: {}", self.cache.directory.display()))?;

        // Validate enabled modules
        for module in self.enabled_modules() {
            match &module.source {
                ModuleSource::Local { path } => {
                    if !path.exists() && !path.is_absolute() {
                        // Try relative to current directory first
                        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                        let cwd_path = cwd.join(path);
                        if cwd_path.exists() {
                            continue;
                        }

                        // Then try relative to config directory
                        let config_path = Self::get_config_path();
                        let default_dir = PathBuf::from(".");
                        let config_dir = config_path.parent().unwrap_or(&default_dir);
                        let full_path = config_dir.join(path);
                        if !full_path.exists() {
                            return Err(anyhow::anyhow!(
                                "Module '{}' local path does not exist: {}",
                                module.name,
                                path.display()
                            ));
                        }
                    }
                }
                ModuleSource::Http { url, .. } => {
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        return Err(anyhow::anyhow!(
                            "Module '{}' has invalid HTTP URL: {}",
                            module.name,
                            url
                        ));
                    }
                }
                ModuleSource::Registry { name, .. } => {
                    if name.is_empty() {
                        return Err(anyhow::anyhow!(
                            "Module '{}' has empty registry name",
                            module.name
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

