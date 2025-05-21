use crate::errors::SkinshiftError;
use crate::firewall::FirewallRule;
use crate::fingerprint::OSFingerprint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// Fingerprint preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintPreset {
    /// Name of the preset
    pub name: String,
    
    /// Description of what this preset mimics
    pub description: String,
    
    /// OS fingerprint configuration
    pub fingerprint: OSFingerprint,
    
    /// Service banners (service name -> banner text)
    pub banners: HashMap<String, String>,
    
    /// Firewall rules
    pub firewall_rules: Option<Vec<FirewallRule>>,
    
    /// Service configurations (service name -> config)
    pub services: HashMap<String, serde_json::Value>,
    
    /// Additional metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl FingerprintPreset {
    /// Create a new fingerprint preset
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        fingerprint: OSFingerprint,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            fingerprint,
            banners: HashMap::new(),
            firewall_rules: None,
            services: HashMap::new(),
            metadata: None,
        }
    }
    
    /// Add a service banner
    pub fn add_banner(&mut self, service: impl Into<String>, banner: impl Into<String>) {
        self.banners.insert(service.into(), banner.into());
    }
    
    /// Add a firewall rule
    pub fn add_firewall_rule(&mut self, rule: FirewallRule) {
        if self.firewall_rules.is_none() {
            self.firewall_rules = Some(Vec::new());
        }
        
        if let Some(rules) = &mut self.firewall_rules {
            rules.push(rule);
        }
    }
    
    /// Add service configuration
    pub fn add_service_config(&mut self, service: impl Into<String>, config: serde_json::Value) {
        self.services.insert(service.into(), config);
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        
        if let Some(metadata) = &mut self.metadata {
            metadata.insert(key.into(), value);
        }
    }
}

/// Manager for handling fingerprint presets
pub struct PresetManager {
    /// Directory containing preset configurations
    presets_dir: PathBuf,
    
    /// Loaded presets
    presets: HashMap<String, FingerprintPreset>,
}

impl PresetManager {
    /// Create a new preset manager
    pub fn new(presets_dir: &str) -> Self {
        Self {
            presets_dir: PathBuf::from(presets_dir),
            presets: HashMap::new(),
        }
    }
    
    /// Initialize the preset manager
    pub async fn init(&self) -> Result<(), SkinshiftError> {
        info!("Initializing preset manager");
        
        // Create the presets directory if it doesn't exist
        if !self.presets_dir.exists() {
            debug!("Creating presets directory: {:?}", self.presets_dir);
            fs::create_dir_all(&self.presets_dir).map_err(|e| {
                SkinshiftError::PresetError(format!("Failed to create presets directory: {}", e))
            })?;
            
            // Create default presets
            self.create_default_presets()?;
        }
        
        debug!("Preset manager initialized");
        Ok(())
    }
    
    /// Load a specific preset
    pub async fn load_preset(&self, name: &str) -> Result<FingerprintPreset, SkinshiftError> {
        info!("Loading preset: {}", name);
        
        let preset_path = self.presets_dir.join(format!("{}.toml", name));
        
        if !preset_path.exists() {
            return Err(SkinshiftError::PresetError(
                format!("Preset not found: {}", name)
            ));
        }
        
        // Read the preset file
        let mut content = String::new();
        {
            let mut file = File::open(&preset_path).map_err(|e| {
                SkinshiftError::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to open preset file: {}", e)
                ))
            })?;
            
            file.read_to_string(&mut content).map_err(|e| {
                SkinshiftError::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read preset file: {}", e)
                ))
            })?;
        }
        
        // Parse the preset
        let preset: FingerprintPreset = toml::from_str(&content).map_err(|e| {
            SkinshiftError::PresetError(format!("Failed to parse preset: {}", e))
        })?;
        
        debug!("Preset loaded successfully: {}", name);
        
        Ok(preset)
    }
    
    /// Load a custom fingerprint configuration
    pub async fn load_custom(&self, path: &Path) -> Result<FingerprintPreset, SkinshiftError> {
        info!("Loading custom fingerprint from: {}", path.display());
        
        if !path.exists() {
            return Err(SkinshiftError::PresetError(
                format!("Custom fingerprint file not found: {}", path.display())
            ));
        }
        
        // Read the custom file
        let mut content = String::new();
        {
            let mut file = File::open(path).map_err(|e| {
                SkinshiftError::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to open custom fingerprint file: {}", e)
                ))
            })?;
            
            file.read_to_string(&mut content).map_err(|e| {
                SkinshiftError::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read custom fingerprint file: {}", e)
                ))
            })?;
        }
        
        // Parse the custom configuration
        let preset: FingerprintPreset = match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => toml::from_str(&content).map_err(|e| {
                SkinshiftError::PresetError(format!("Failed to parse TOML: {}", e))
            })?,
            Some("json") => serde_json::from_str(&content).map_err(|e| {
                SkinshiftError::PresetError(format!("Failed to parse JSON: {}", e))
            })?,
            _ => return Err(SkinshiftError::PresetError(
                format!("Unsupported file format: {}", path.display())
            )),
        };
        
        debug!("Custom fingerprint loaded successfully");
        
        Ok(preset)
    }
    
    /// List available presets
    pub async fn list_presets(&self) -> Result<Vec<String>, SkinshiftError> {
        info!("Listing available presets");
        
        if !self.presets_dir.exists() {
            debug!("Presets directory does not exist");
            return Ok(Vec::new());
        }
        
        let mut presets = Vec::new();
        
        for entry in fs::read_dir(&self.presets_dir).map_err(|e| {
            SkinshiftError::IOError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read presets directory: {}", e)
            ))
        })? {
            let entry = entry.map_err(|e| {
                SkinshiftError::IOError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read directory entry: {}", e)
                ))
            })?;
            
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    presets.push(stem.to_string());
                }
            }
        }
        
        debug!("Found {} presets", presets.len());
        
        Ok(presets)
    }
    
    /// Create default presets
    fn create_default_presets(&self) -> Result<(), SkinshiftError> {
        info!("Creating default presets");
        
        // Create Windows Server 2019 preset
        let windows_2019 = FingerprintPreset::new(
            "windows_server2019",
            "Windows Server 2019",
            OSFingerprint::windows(Some("Server 2019".to_string())),
        );
        
        self.save_preset(&windows_2019)?;
        
        // Create Linux Standard preset
        let mut linux_standard = FingerprintPreset::new(
            "linux_standard",
            "Standard Linux Server",
            OSFingerprint::linux(Some("4.19".to_string())),
        );
        
        // Add SSH banner
        linux_standard.add_banner("ssh", "SSH-2.0-OpenSSH_7.9p1 Debian-10+deb10u2");
        
        self.save_preset(&linux_standard)?;
        
        // Create Router preset
        let router = FingerprintPreset::new(
            "router_vulnerable",
            "Vulnerable Router",
            OSFingerprint::router("Generic Router"),
        );
        
        self.save_preset(&router)?;
        
        // Create Minimal preset
        let minimal = FingerprintPreset::new(
            "silent_minimal",
            "Minimal Stealth Profile",
            OSFingerprint::minimal(),
        );
        
        self.save_preset(&minimal)?;
        
        // Create Random/Changing preset
        let random = FingerprintPreset::new(
            "random_changing",
            "Random Changing Profile",
            OSFingerprint::minimal(),
        );
        
        self.save_preset(&random)?;
        
        info!("Default presets created successfully");
        
        Ok(())
    }
    
    /// Save a preset to file
    fn save_preset(&self, preset: &FingerprintPreset) -> Result<(), SkinshiftError> {
        debug!("Saving preset: {}", preset.name);
        
        let preset_path = self.presets_dir.join(format!("{}.toml", preset.name));
        
        // Serialize the preset
        let content = toml::to_string_pretty(preset).map_err(|e| {
            SkinshiftError::PresetError(format!("Failed to serialize preset: {}", e))
        })?;
        
        // Write to file
        fs::write(&preset_path, content).map_err(|e| {
            SkinshiftError::IOError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to write preset file: {}", e)
            ))
        })?;
        
        debug!("Preset saved successfully: {}", preset.name);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_fingerprint_preset() {
        let mut preset = FingerprintPreset::new(
            "test",
            "Test Preset",
            OSFingerprint::windows(Some("10".to_string())),
        );
        
        preset.add_banner("ssh", "SSH-2.0-OpenSSH_8.2p1");
        preset.add_service_config("http", serde_json::json!({
            "port": 80,
            "enabled": true,
        }));
        
        assert_eq!(preset.name, "test");
        assert_eq!(preset.description, "Test Preset");
        assert_eq!(preset.banners.get("ssh"), Some(&"SSH-2.0-OpenSSH_8.2p1".to_string()));
        assert_eq!(preset.services.get("http").and_then(|v| v.get("port")).and_then(|v| v.as_u64()), Some(80));
    }
    
    #[tokio::test]
    async fn test_preset_manager() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap();
        
        let manager = PresetManager::new(dir_path);
        
        // Initialize the manager (creates default presets)
        manager.init().await.unwrap();
        
        // List presets
        let presets = manager.list_presets().await.unwrap();
        assert!(presets.len() > 0);
        
        // Load a preset
        if let Some(preset_name) = presets.first() {
            let preset = manager.load_preset(preset_name).await.unwrap();
            assert_eq!(&preset.name, preset_name);
        }
    }
}
