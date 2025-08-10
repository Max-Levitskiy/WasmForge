use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::config::{Config, ModuleConfig, ModuleSource};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub checksum: String,
    pub size_bytes: u64,
    pub cached_at: u64,
    pub source: ModuleSource,
    pub wasm_path: PathBuf,
}

pub struct ModuleManager {
    config: Config,
    cache_dir: PathBuf,
    loaded_modules: HashMap<String, ModuleMetadata>,
    client: reqwest::Client,
}

impl ModuleManager {
    pub fn new(config: Config) -> Result<Self> {
        let cache_dir = config.cache.directory.clone();
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

        Ok(Self {
            config,
            cache_dir,
            loaded_modules: HashMap::new(),
            client: reqwest::Client::new(),
        })
    }

    pub async fn load_all_modules(&mut self) -> Result<()> {
        let enabled_modules: Vec<_> = self.config.enabled_modules().cloned().collect();
        for module_config in enabled_modules {
            match self.load_module(&module_config).await {
                Ok(metadata) => {
                    println!("✓ Loaded module: {} v{}", metadata.name, metadata.version);
                    self.loaded_modules.insert(metadata.name.clone(), metadata);
                }
                Err(e) => {
                    eprintln!("✗ Failed to load module '{}': {}", module_config.name, e);
                    // Continue loading other modules instead of failing completely
                }
            }
        }
        Ok(())
    }

    pub async fn load_module(&mut self, module_config: &ModuleConfig) -> Result<ModuleMetadata> {
        match &module_config.source {
            ModuleSource::Local { path } => self.load_local_module(module_config, path).await,
            ModuleSource::Http { url, checksum } => {
                self.load_http_module(module_config, url, checksum.as_deref()).await
            }
            ModuleSource::Registry { name, version } => {
                self.load_registry_module(module_config, name, version.as_deref()).await
            }
        }
    }

    async fn load_local_module(&self, config: &ModuleConfig, path: &Path) -> Result<ModuleMetadata> {
        let wasm_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            // Try relative to current directory first, then relative to config directory
            if path.exists() {
                path.to_path_buf()
            } else {
                let config_path = Config::get_config_path();
                let default_dir = PathBuf::from(".");
                let config_dir = config_path.parent().unwrap_or(&default_dir);
                config_dir.join(path)
            }
        };

        if !wasm_path.exists() {
            return Err(anyhow::anyhow!(
                "Local module file not found: {}",
                wasm_path.display()
            ));
        }

        let wasm_bytes = fs::read(&wasm_path)
            .with_context(|| format!("Failed to read module file: {}", wasm_path.display()))?;

        self.validate_wasm_module(&wasm_bytes)?;

        let checksum = self.calculate_checksum(&wasm_bytes);
        let size_bytes = wasm_bytes.len() as u64;

        Ok(ModuleMetadata {
            id: Uuid::new_v4().to_string(),
            name: config.name.clone(),
            version: config.version.clone().unwrap_or_else(|| "unknown".to_string()),
            description: config.description.clone().unwrap_or_default(),
            checksum,
            size_bytes,
            cached_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            source: config.source.clone(),
            wasm_path,
        })
    }

    async fn load_http_module(
        &self,
        config: &ModuleConfig,
        url: &str,
        expected_checksum: Option<&str>,
    ) -> Result<ModuleMetadata> {
        let module_id = format!("{}_{}", config.name, 
            expected_checksum.unwrap_or(&format!("{:x}", md5::compute(url))));
        let cached_path = self.cache_dir.join(format!("{}.wasm", module_id));

        // Check if module is already cached and valid
        if let Ok(metadata) = self.load_cached_metadata(&module_id) {
            if cached_path.exists() && self.is_cache_valid(&metadata) {
                println!("Using cached module: {}", config.name);
                return Ok(metadata);
            }
        }

        println!("Downloading module from: {}", url);
        let response = self.client.get(url).send().await
            .with_context(|| format!("Failed to download module from: {}", url))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP error downloading module: {} - {}",
                response.status(),
                url
            ));
        }

        let wasm_bytes = response.bytes().await
            .context("Failed to read response body")?;

        let checksum = self.calculate_checksum(&wasm_bytes);

        // Verify checksum if provided
        if let Some(expected) = expected_checksum {
            if checksum != expected {
                return Err(anyhow::anyhow!(
                    "Checksum mismatch for module '{}': expected {}, got {}",
                    config.name,
                    expected,
                    checksum
                ));
            }
        }

        self.validate_wasm_module(&wasm_bytes)?;

        // Save to cache
        fs::write(&cached_path, &wasm_bytes)
            .with_context(|| format!("Failed to cache module: {}", cached_path.display()))?;

        let metadata = ModuleMetadata {
            id: module_id.clone(),
            name: config.name.clone(),
            version: config.version.clone().unwrap_or_else(|| "unknown".to_string()),
            description: config.description.clone().unwrap_or_default(),
            checksum,
            size_bytes: wasm_bytes.len() as u64,
            cached_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            source: config.source.clone(),
            wasm_path: cached_path,
        };

        // Save metadata
        self.save_cached_metadata(&metadata)?;

        Ok(metadata)
    }

    async fn load_registry_module(
        &self,
        _config: &ModuleConfig,
        _name: &str,
        _version: Option<&str>,
    ) -> Result<ModuleMetadata> {
        // TODO: Implement registry module loading when we have a registry
        Err(anyhow::anyhow!("Registry module loading not yet implemented"))
    }

    fn validate_wasm_module(&self, wasm_bytes: &[u8]) -> Result<()> {
        // Basic WASM validation - check for WASM magic bytes
        if wasm_bytes.len() < 8 {
            return Err(anyhow::anyhow!("Invalid WASM module: too short"));
        }

        if &wasm_bytes[0..4] != b"\0asm" {
            return Err(anyhow::anyhow!("Invalid WASM module: missing magic bytes"));
        }

        // Check version (should be 1)
        if &wasm_bytes[4..8] != [0x01, 0x00, 0x00, 0x00] {
            return Err(anyhow::anyhow!("Unsupported WASM version"));
        }

        Ok(())
    }

    fn calculate_checksum(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn get_metadata_path(&self, module_id: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", module_id))
    }

    fn load_cached_metadata(&self, module_id: &str) -> Result<ModuleMetadata> {
        let metadata_path = self.get_metadata_path(module_id);
        let contents = fs::read_to_string(&metadata_path)
            .context("Failed to read cached metadata")?;
        let metadata: ModuleMetadata = serde_json::from_str(&contents)
            .context("Failed to parse cached metadata")?;
        Ok(metadata)
    }

    fn save_cached_metadata(&self, metadata: &ModuleMetadata) -> Result<()> {
        let metadata_path = self.get_metadata_path(&metadata.id);
        let contents = serde_json::to_string_pretty(metadata)
            .context("Failed to serialize metadata")?;
        fs::write(&metadata_path, contents)
            .context("Failed to write metadata to cache")?;
        Ok(())
    }

    fn is_cache_valid(&self, metadata: &ModuleMetadata) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let ttl_seconds = self.config.cache.ttl_hours * 3600;
        
        (now - metadata.cached_at) < ttl_seconds
    }

    pub fn get_loaded_modules(&self) -> &HashMap<String, ModuleMetadata> {
        &self.loaded_modules
    }

    pub fn get_module_metadata(&self, name: &str) -> Option<&ModuleMetadata> {
        self.loaded_modules.get(name)
    }

    pub fn get_module_bytes(&self, name: &str) -> Result<Vec<u8>> {
        let metadata = self.loaded_modules.get(name)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not loaded", name))?;

        fs::read(&metadata.wasm_path)
            .with_context(|| format!("Failed to read module file: {}", metadata.wasm_path.display()))
    }

    pub async fn reload_module(&mut self, name: &str) -> Result<()> {
        let module_config = self.config.find_module(name)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not found in configuration", name))?
            .clone();

        match self.load_module(&module_config).await {
            Ok(metadata) => {
                println!("✓ Reloaded module: {} v{}", metadata.name, metadata.version);
                self.loaded_modules.insert(metadata.name.clone(), metadata);
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Failed to reload module '{}': {}", name, e);
                Err(e)
            }
        }
    }

    pub fn cleanup_cache(&self) -> Result<()> {
        // TODO: Implement cache cleanup based on size limits and TTL
        println!("Cache cleanup not yet implemented");
        Ok(())
    }
}

