use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct CamaleonConfig {
    pub general: GeneralConfig,
    pub skinshift: SkinshiftConfig,
    pub eye360: Eye360Config,
    pub nettongue: NettongueConfig,
    pub lurefield: LurefieldConfig,
    pub posture: PostureConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub log_level: String,
    pub adaptive_mode: bool,
    pub default_posture: String,
}

#[derive(Debug, Deserialize)]
pub struct SkinshiftConfig {
    pub enabled: bool,
    pub presets_dir: String,
    pub rotation_interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct Eye360Config {
    pub enabled: bool,
    pub syscall_monitoring: bool,
    pub log_suspicious: bool,
    pub ebpf_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct NettongueConfig {
    pub enabled: bool,
    pub pcap_enabled: bool,
    pub interface: String,
    pub latency_fuzz_enabled: bool,
    pub latency_fuzz_min_ms: u64,
    pub latency_fuzz_max_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct LurefieldConfig {
    pub enabled: bool,
    pub honeypot_dir: String,
    pub max_honeypots: u32,
    pub auto_deploy: bool,
}

#[derive(Debug, Deserialize)]
pub struct PostureConfig {
    pub change_threshold: f64,
    pub service_rotation_enabled: bool,
    pub service_rotation_interval: u64,
    pub postures: Vec<String>,
}

impl CamaleonConfig {
    pub fn load(config_path: Option<&Path>) -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            // Start with default config
            .add_source(File::with_name("config/default"))
            // Add environment variables with prefix CAMALEON
            .add_source(Environment::with_prefix("CAMALEON").separator("__"));

        // Add config path if specified
        if let Some(path) = config_path {
            builder = builder.add_source(File::from(path));
        }

        // Build and deserialize
        let config = builder.build()?;
        config.try_deserialize()
    }
}

pub fn init_config(config_path: Option<&Path>) -> Result<CamaleonConfig> {
    let config = CamaleonConfig::load(config_path)?;
    Ok(config)
}
