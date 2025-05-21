use crate::errors::SkinshiftError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, error, info, warn};

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,
    
    /// Whether the service is enabled
    pub enabled: bool,
    
    /// Port number
    pub port: Option<u16>,
    
    /// Configuration options
    pub options: HashMap<String, serde_json::Value>,
}

impl ServiceConfig {
    /// Create a new service configuration
    pub fn new(name: impl Into<String>, enabled: bool) -> Self {
        Self {
            name: name.into(),
            enabled,
            port: None,
            options: HashMap::new(),
        }
    }
    
    /// Set the port number
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    /// Add an option
    pub fn with_option(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.options.insert(key.into(), value);
        self
    }
}

/// Manager for handling service configurations
pub struct ServiceManager {
    /// Map of service names to their original configurations
    original_configs: HashMap<String, ServiceConfig>,
    
    /// Map of service names to their current configurations
    current_configs: HashMap<String, ServiceConfig>,
}

impl ServiceManager {
    /// Create a new service manager
    pub fn new() -> Self {
        Self {
            original_configs: HashMap::new(),
            current_configs: HashMap::new(),
        }
    }
    
    /// Configure a service
    pub async fn configure_service(&self, service_name: &str, config: &serde_json::Value) -> Result<(), SkinshiftError> {
        info!("Configuring service: {}", service_name);
        
        // Parse the configuration
        let service_config = self.parse_service_config(service_name, config)?;
        
        // Apply the configuration
        self.apply_service_config(&service_config).await?;
        
        debug!("Service configured successfully: {}", service_name);
        
        Ok(())
    }
    
    /// Reset all service configurations
    pub async fn reset_all(&self) -> Result<(), SkinshiftError> {
        info!("Resetting all service configurations");
        
        for (service_name, config) in &self.original_configs {
            info!("Resetting service: {}", service_name);
            
            // Apply the original configuration
            if let Err(e) = self.apply_service_config(config).await {
                warn!("Failed to reset service configuration for {}: {}", service_name, e);
                // Continue with other services
            }
        }
        
        debug!("All service configurations reset successfully");
        
        Ok(())
    }
    
    /// Parse a service configuration
    fn parse_service_config(&self, service_name: &str, config: &serde_json::Value) -> Result<ServiceConfig, SkinshiftError> {
        debug!("Parsing service configuration for: {}", service_name);
        
        // Try to deserialize the configuration
        let service_config: ServiceConfig = match serde_json::from_value(config.clone()) {
            Ok(config) => config,
            Err(e) => {
                // If deserialization fails, try to create a minimal config
                warn!("Failed to parse service configuration: {}", e);
                
                let enabled = config.get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                    
                let port = config.get("port")
                    .and_then(|v| v.as_u64())
                    .map(|p| p as u16);
                    
                let mut service_config = ServiceConfig::new(service_name, enabled);
                
                if let Some(port) = port {
                    service_config.port = Some(port);
                }
                
                service_config
            }
        };
        
        Ok(service_config)
    }
    
    /// Apply a service configuration
    async fn apply_service_config(&self, config: &ServiceConfig) -> Result<(), SkinshiftError> {
        let service_name = &config.name;
        debug!("Applying service configuration for: {}", service_name);
        
        // Store the original configuration if we don't have it yet
        if !self.original_configs.contains_key(service_name) {
            // In a real implementation, we would fetch the current configuration
            // and store it. For now, just simulate success.
            debug!("Storing original configuration for: {}", service_name);
        }
        
        // Enable or disable the service
        if config.enabled {
            self.enable_service(service_name).await?;
        } else {
            self.disable_service(service_name).await?;
        }
        
        // Configure the port if specified
        if let Some(port) = config.port {
            self.configure_service_port(service_name, port).await?;
        }
        
        // Apply additional options
        for (key, value) in &config.options {
            self.configure_service_option(service_name, key, value).await?;
        }
        
        // In a real implementation, we would store the current configuration
        // self.current_configs.insert(service_name.clone(), config.clone());
        
        debug!("Service configuration applied successfully: {}", service_name);
        
        Ok(())
    }
    
    /// Enable a service
    async fn enable_service(&self, service_name: &str) -> Result<(), SkinshiftError> {
        debug!("Enabling service: {}", service_name);
        
        // This would be done with systemctl or similar
        // For now, just simulate success
        debug!("Service enabled (simulated): {}", service_name);
        
        Ok(())
    }
    
    /// Disable a service
    async fn disable_service(&self, service_name: &str) -> Result<(), SkinshiftError> {
        debug!("Disabling service: {}", service_name);
        
        // This would be done with systemctl or similar
        // For now, just simulate success
        debug!("Service disabled (simulated): {}", service_name);
        
        Ok(())
    }
    
    /// Configure a service port
    async fn configure_service_port(&self, service_name: &str, port: u16) -> Result<(), SkinshiftError> {
        debug!("Configuring port for service {}: {}", service_name, port);
        
        // This would modify the service configuration files
        // For now, just simulate success
        debug!("Port configured (simulated) for service {}: {}", service_name, port);
        
        Ok(())
    }
    
    /// Configure a service option
    async fn configure_service_option(&self, service_name: &str, key: &str, value: &serde_json::Value) -> Result<(), SkinshiftError> {
        debug!("Configuring option for service {}: {} = {:?}", service_name, key, value);
        
        // This would modify the service configuration files
        // For now, just simulate success
        debug!("Option configured (simulated) for service {}: {} = {:?}", service_name, key, value);
        
        Ok(())
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_service_config() {
        let config = ServiceConfig::new("test", true)
            .with_port(8080)
            .with_option("max_connections", serde_json::json!(100));
        
        assert_eq!(config.name, "test");
        assert_eq!(config.enabled, true);
        assert_eq!(config.port, Some(8080));
        assert_eq!(config.options.get("max_connections").and_then(|v| v.as_u64()), Some(100));
    }
    
    #[test]
    fn test_parse_service_config() {
        let manager = ServiceManager::new();
        
        let config = serde_json::json!({
            "enabled": true,
            "port": 8080,
            "options": {
                "max_connections": 100
            }
        });
        
        let service_config = manager.parse_service_config("test", &config).unwrap();
        
        assert_eq!(service_config.name, "test");
        assert_eq!(service_config.enabled, true);
        assert_eq!(service_config.port, Some(8080));
    }
}
