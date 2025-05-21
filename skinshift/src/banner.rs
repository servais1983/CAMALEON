use crate::errors::SkinshiftError;
use regex::Regex;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Banner modification configuration
#[derive(Debug, Clone)]
pub struct BannerConfig {
    /// Service name (e.g., "ssh", "http", "smtp")
    pub service_name: String,
    
    /// Banner text to display
    pub banner_text: String,
    
    /// Configuration file path
    pub config_path: Option<String>,
    
    /// Regex pattern for finding the banner in the config file
    pub pattern: Option<String>,
    
    /// Whether to replace or append the banner
    pub replace: bool,
}

impl BannerConfig {
    /// Create a new banner configuration
    pub fn new(service_name: impl Into<String>, banner_text: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            banner_text: banner_text.into(),
            config_path: None,
            pattern: None,
            replace: true,
        }
    }
    
    /// Set the configuration file path
    pub fn with_config_path(mut self, path: impl Into<String>) -> Self {
        self.config_path = Some(path.into());
        self
    }
    
    /// Set the regex pattern for finding the banner
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }
    
    /// Set whether to replace or append the banner
    pub fn with_replace(mut self, replace: bool) -> Self {
        self.replace = replace;
        self
    }
}

/// Manager for handling service banner changes
pub struct BannerManager {
    /// Map of service names to their original banners
    original_banners: HashMap<String, String>,
    
    /// Map of service names to their current banners
    current_banners: HashMap<String, String>,
    
    /// Service config paths (for reverting changes)
    service_configs: HashMap<String, String>,
}

impl BannerManager {
    /// Create a new banner manager
    pub fn new() -> Self {
        Self {
            original_banners: HashMap::new(),
            current_banners: HashMap::new(),
            service_configs: HashMap::new(),
        }
    }
    
    /// Initialize the banner manager
    pub async fn init(&self) -> Result<(), SkinshiftError> {
        info!("Initializing banner manager");
        
        // We could collect information about installed services here
        debug!("Banner manager initialized");
        
        Ok(())
    }
    
    /// Set a banner for a service
    pub async fn set_banner(&self, service_name: &str, banner: &str) -> Result<(), SkinshiftError> {
        info!("Setting banner for service: {}", service_name);
        
        // Determine the appropriate configuration based on the service
        let config = match service_name.to_lowercase().as_str() {
            "ssh" => self.get_ssh_config(banner)?,
            "http" | "apache" | "nginx" => self.get_http_config(service_name, banner)?,
            "ftp" => self.get_ftp_config(banner)?,
            "smtp" => self.get_smtp_config(banner)?,
            "telnet" => self.get_telnet_config(banner)?,
            _ => {
                warn!("Unknown service: {}, cannot set banner", service_name);
                return Err(SkinshiftError::BannerError(
                    format!("Unknown service: {}", service_name)
                ));
            }
        };
        
        // Apply the banner configuration
        self.apply_banner_config(&config).await?;
        
        debug!("Banner set successfully for {}", service_name);
        Ok(())
    }
    
    /// Reset all banners to their original values
    pub async fn reset_all(&self) -> Result<(), SkinshiftError> {
        info!("Resetting all banners to original values");
        
        for (service, original_banner) in &self.original_banners {
            info!("Resetting banner for service: {}", service);
            
            // Build a config for the original banner
            let config = match service.to_lowercase().as_str() {
                "ssh" => self.get_ssh_config(original_banner)?,
                "http" | "apache" | "nginx" => self.get_http_config(service, original_banner)?,
                "ftp" => self.get_ftp_config(original_banner)?,
                "smtp" => self.get_smtp_config(original_banner)?,
                "telnet" => self.get_telnet_config(original_banner)?,
                _ => {
                    warn!("Unknown service: {}, cannot reset banner", service);
                    continue;
                }
            };
            
            // Apply the original banner
            if let Err(e) = self.apply_banner_config(&config).await {
                warn!("Failed to reset banner for {}: {}", service, e);
                // Continue with other services
            }
        }
        
        debug!("All banners reset successfully");
        Ok(())
    }
    
    /// Reset a specific service's banner
    pub async fn reset_banner(&self, service_name: &str) -> Result<(), SkinshiftError> {
        info!("Resetting banner for service: {}", service_name);
        
        if let Some(original_banner) = self.original_banners.get(service_name) {
            // Build a config for the original banner
            let config = match service_name.to_lowercase().as_str() {
                "ssh" => self.get_ssh_config(original_banner)?,
                "http" | "apache" | "nginx" => self.get_http_config(service_name, original_banner)?,
                "ftp" => self.get_ftp_config(original_banner)?,
                "smtp" => self.get_smtp_config(original_banner)?,
                "telnet" => self.get_telnet_config(original_banner)?,
                _ => {
                    return Err(SkinshiftError::BannerError(
                        format!("Unknown service: {}", service_name)
                    ));
                }
            };
            
            // Apply the original banner
            self.apply_banner_config(&config).await?;
            
            debug!("Banner reset successfully for {}", service_name);
            Ok(())
        } else {
            warn!("No original banner stored for service: {}", service_name);
            Err(SkinshiftError::BannerError(
                format!("No original banner stored for service: {}", service_name)
            ))
        }
    }
    
    /// Apply a banner configuration
    async fn apply_banner_config(&self, config: &BannerConfig) -> Result<(), SkinshiftError> {
        debug!("Applying banner config for {}", config.service_name);
        
        if let Some(config_path) = &config.config_path {
            let path = Path::new(config_path);
            
            // Ensure the path exists
            if !path.exists() {
                return Err(SkinshiftError::BannerError(
                    format!("Config file does not exist: {}", config_path)
                ));
            }
            
            // Read the current file
            let mut content = String::new();
            {
                let mut file = File::open(path).map_err(|e| {
                    SkinshiftError::IOError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to open config file: {}", e)
                    ))
                })?;
                
                file.read_to_string(&mut content).map_err(|e| {
                    SkinshiftError::IOError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to read config file: {}", e)
                    ))
                })?;
            }
            
            // Create a backup if we don't have the original yet
            let service_name = &config.service_name;
            if !self.original_banners.contains_key(service_name) {
                // TODO: Store the original banner
                // self.original_banners.insert(service_name.clone(), original_banner);
                // self.service_configs.insert(service_name.clone(), config_path.clone());
            }
            
            // Modify the content based on the pattern
            let new_content = if let Some(pattern) = &config.pattern {
                let re = Regex::new(pattern).map_err(|e| {
                    SkinshiftError::BannerError(
                        format!("Invalid regex pattern: {}", e)
                    )
                })?;
                
                if config.replace {
                    re.replace_all(&content, config.banner_text.as_str()).to_string()
                } else {
                    content + &config.banner_text
                }
            } else {
                // No pattern, just append
                content + &config.banner_text
            };
            
            // Write the new content
            {
                let mut file = File::create(path).map_err(|e| {
                    SkinshiftError::IOError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to create config file: {}", e)
                    ))
                })?;
                
                file.write_all(new_content.as_bytes()).map_err(|e| {
                    SkinshiftError::IOError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to write to config file: {}", e)
                    ))
                })?;
            }
            
            // Update the current banner
            // self.current_banners.insert(service_name.clone(), config.banner_text.clone());
            
            debug!("Banner applied successfully for {}", service_name);
        } else {
            warn!("No config path specified for service: {}", config.service_name);
            // This might be a service where the banner is set in memory
            // For now, just simulate success
            debug!("Banner applied in-memory (simulated) for {}", config.service_name);
        }
        
        Ok(())
    }
    
    /// Get SSH banner configuration
    fn get_ssh_config(&self, banner: &str) -> Result<BannerConfig, SkinshiftError> {
        let config_path = "/etc/ssh/sshd_config";
        
        Ok(BannerConfig::new("ssh", banner)
            .with_config_path(config_path)
            .with_pattern(r"^#?Banner\s+.*$")
            .with_replace(true))
    }
    
    /// Get HTTP server banner configuration
    fn get_http_config(&self, server_type: &str, banner: &str) -> Result<BannerConfig, SkinshiftError> {
        match server_type.to_lowercase().as_str() {
            "apache" => {
                let config_path = "/etc/apache2/apache2.conf";
                
                Ok(BannerConfig::new("http", banner)
                    .with_config_path(config_path)
                    .with_pattern(r"^ServerTokens\s+.*$")
                    .with_replace(true))
            }
            "nginx" => {
                let config_path = "/etc/nginx/nginx.conf";
                
                Ok(BannerConfig::new("http", banner)
                    .with_config_path(config_path)
                    .with_pattern(r"^server_tokens\s+.*$")
                    .with_replace(true))
            }
            _ => {
                // Generic HTTP server
                Ok(BannerConfig::new("http", banner))
            }
        }
    }
    
    /// Get FTP banner configuration
    fn get_ftp_config(&self, banner: &str) -> Result<BannerConfig, SkinshiftError> {
        // Try to find vsftpd config
        let config_path = "/etc/vsftpd.conf";
        
        Ok(BannerConfig::new("ftp", banner)
            .with_config_path(config_path)
            .with_pattern(r"^#?ftpd_banner=.*$")
            .with_replace(true))
    }
    
    /// Get SMTP banner configuration
    fn get_smtp_config(&self, banner: &str) -> Result<BannerConfig, SkinshiftError> {
        // Try to find postfix config
        let config_path = "/etc/postfix/main.cf";
        
        Ok(BannerConfig::new("smtp", banner)
            .with_config_path(config_path)
            .with_pattern(r"^#?smtpd_banner\s*=.*$")
            .with_replace(true))
    }
    
    /// Get Telnet banner configuration
    fn get_telnet_config(&self, banner: &str) -> Result<BannerConfig, SkinshiftError> {
        // Try to find telnet config
        let config_path = "/etc/issue.net";
        
        Ok(BannerConfig::new("telnet", banner)
            .with_config_path(config_path)
            .with_replace(true))
    }
}

impl Default for BannerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_banner_config() {
        let config = BannerConfig::new("test", "Test Banner")
            .with_config_path("/etc/test.conf")
            .with_pattern(r"^Banner\s+.*$")
            .with_replace(true);
        
        assert_eq!(config.service_name, "test");
        assert_eq!(config.banner_text, "Test Banner");
        assert_eq!(config.config_path, Some("/etc/test.conf".to_string()));
        assert_eq!(config.pattern, Some(r"^Banner\s+.*$".to_string()));
        assert_eq!(config.replace, true);
    }
}
