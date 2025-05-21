use crate::errors::SkinshiftError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, error, info, warn};

/// Firewall rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    /// Rule name/description
    pub name: String,
    
    /// Protocol (tcp, udp, icmp, all)
    pub protocol: String,
    
    /// Source IP/network (CIDR format)
    pub source: Option<String>,
    
    /// Source port
    pub source_port: Option<String>,
    
    /// Destination IP/network (CIDR format)
    pub destination: Option<String>,
    
    /// Destination port
    pub destination_port: Option<String>,
    
    /// Action (accept, drop, reject)
    pub action: String,
    
    /// Rule priority (higher = more important)
    pub priority: Option<u32>,
    
    /// Additional options
    pub options: Option<HashMap<String, String>>,
}

impl FirewallRule {
    /// Create a new firewall rule
    pub fn new(
        name: impl Into<String>,
        protocol: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            protocol: protocol.into(),
            source: None,
            source_port: None,
            destination: None,
            destination_port: None,
            action: action.into(),
            priority: None,
            options: None,
        }
    }
    
    /// Set the source IP/network
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
    
    /// Set the source port
    pub fn with_source_port(mut self, port: impl Into<String>) -> Self {
        self.source_port = Some(port.into());
        self
    }
    
    /// Set the destination IP/network
    pub fn with_destination(mut self, destination: impl Into<String>) -> Self {
        self.destination = Some(destination.into());
        self
    }
    
    /// Set the destination port
    pub fn with_destination_port(mut self, port: impl Into<String>) -> Self {
        self.destination_port = Some(port.into());
        self
    }
    
    /// Set the priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = Some(priority);
        self
    }
    
    /// Add an option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if self.options.is_none() {
            self.options = Some(HashMap::new());
        }
        
        if let Some(options) = &mut self.options {
            options.insert(key.into(), value.into());
        }
        
        self
    }
    
    /// Convert to iptables command arguments
    pub fn to_iptables_args(&self) -> Vec<String> {
        let mut args = vec!["-A".to_string(), "INPUT".to_string()];
        
        // Protocol
        args.push("-p".to_string());
        args.push(self.protocol.clone());
        
        // Source IP/network
        if let Some(source) = &self.source {
            args.push("-s".to_string());
            args.push(source.clone());
        }
        
        // Source port
        if let Some(sport) = &self.source_port {
            args.push("--sport".to_string());
            args.push(sport.clone());
        }
        
        // Destination IP/network
        if let Some(destination) = &self.destination {
            args.push("-d".to_string());
            args.push(destination.clone());
        }
        
        // Destination port
        if let Some(dport) = &self.destination_port {
            args.push("--dport".to_string());
            args.push(dport.clone());
        }
        
        // Action
        args.push("-j".to_string());
        args.push(self.action.to_uppercase());
        
        // Add comment with rule name
        args.push("-m".to_string());
        args.push("comment".to_string());
        args.push("--comment".to_string());
        args.push(format!("CAMALEON: {}", self.name));
        
        args
    }
}

/// Manager for handling firewall rules
pub struct FirewallManager {
    /// Whether iptables is available
    has_iptables: bool,
    
    /// Whether we have superuser privileges
    has_superuser: bool,
    
    /// Original firewall rules (for restoration)
    original_rules: Vec<String>,
    
    /// Currently active custom rules
    active_rules: Vec<FirewallRule>,
}

impl FirewallManager {
    /// Create a new firewall manager
    pub async fn new() -> Result<Self, SkinshiftError> {
        // Check for iptables
        let has_iptables = Self::check_iptables();
        
        // Check for superuser privileges
        let has_superuser = Self::check_superuser();
        
        if !has_iptables {
            warn!("iptables not found, firewall functionality will be limited");
        }
        
        if !has_superuser {
            warn!("Superuser privileges not available, firewall functionality will be limited");
        }
        
        let original_rules = if has_iptables && has_superuser {
            Self::backup_rules()?
        } else {
            Vec::new()
        };
        
        Ok(Self {
            has_iptables,
            has_superuser,
            original_rules,
            active_rules: Vec::new(),
        })
    }
    
    /// Apply a set of firewall rules
    pub async fn apply_rules(&self, rules: &[FirewallRule]) -> Result<(), SkinshiftError> {
        info!("Applying {} firewall rules", rules.len());
        
        if !self.has_iptables || !self.has_superuser {
            warn!("Firewall functionality limited, simulating rule application");
            for rule in rules {
                debug!("Would apply rule: {:?}", rule);
            }
            return Ok(());
        }
        
        // Add CAMALEON chain if it doesn't exist
        self.ensure_camaleon_chain()?;
        
        // Apply each rule
        for rule in rules {
            debug!("Applying rule: {:?}", rule);
            
            let args = rule.to_iptables_args();
            
            let output = Command::new("iptables")
                .args(&args)
                .output();
                
            match output {
                Ok(output) => {
                    if !output.status.success() {
                        let error = String::from_utf8_lossy(&output.stderr);
                        warn!("Failed to apply rule '{}': {}", rule.name, error);
                        // Continue with other rules
                    } else {
                        debug!("Rule applied successfully: {}", rule.name);
                    }
                }
                Err(e) => {
                    error!("Error executing iptables: {}", e);
                    return Err(SkinshiftError::FirewallError(
                        format!("Error executing iptables: {}", e)
                    ));
                }
            }
        }
        
        info!("Firewall rules applied successfully");
        
        Ok(())
    }
    
    /// Reset firewall rules to original state
    pub async fn reset(&self) -> Result<(), SkinshiftError> {
        info!("Resetting firewall rules to original state");
        
        if !self.has_iptables || !self.has_superuser {
            debug!("Firewall functionality limited, simulating rule reset");
            return Ok(());
        }
        
        // Clear CAMALEON-specific rules
        self.clear_camaleon_rules()?;
        
        info!("Firewall rules reset successfully");
        
        Ok(())
    }
    
    /// Check if iptables is available
    fn check_iptables() -> bool {
        let output = Command::new("which")
            .arg("iptables")
            .output();
            
        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    /// Check if we have superuser privileges
    fn check_superuser() -> bool {
        #[cfg(target_family = "unix")]
        {
            unsafe { libc::geteuid() == 0 }
        }
        
        #[cfg(not(target_family = "unix"))]
        {
            false
        }
    }
    
    /// Backup current firewall rules
    fn backup_rules() -> Result<Vec<String>, SkinshiftError> {
        debug!("Backing up current firewall rules");
        
        let output = Command::new("iptables-save")
            .output()
            .map_err(|e| {
                SkinshiftError::FirewallError(format!("Failed to backup rules: {}", e))
            })?;
            
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(SkinshiftError::FirewallError(
                format!("Failed to backup rules: {}", error)
            ));
        }
        
        let rules = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(String::from)
            .collect();
            
        Ok(rules)
    }
    
    /// Ensure the CAMALEON chain exists
    fn ensure_camaleon_chain(&self) -> Result<(), SkinshiftError> {
        debug!("Ensuring CAMALEON chain exists");
        
        // Check if the chain already exists
        let output = Command::new("iptables")
            .args(&["-L", "CAMALEON"])
            .output();
            
        match output {
            Ok(output) => {
                if output.status.success() {
                    debug!("CAMALEON chain already exists");
                    return Ok(());
                }
            }
            Err(_) => {}
        }
        
        // Create the chain
        let output = Command::new("iptables")
            .args(&["-N", "CAMALEON"])
            .output()
            .map_err(|e| {
                SkinshiftError::FirewallError(format!("Failed to create CAMALEON chain: {}", e))
            })?;
            
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(SkinshiftError::FirewallError(
                format!("Failed to create CAMALEON chain: {}", error)
            ));
        }
        
        // Add a jump to the CAMALEON chain from INPUT
        let output = Command::new("iptables")
            .args(&["-I", "INPUT", "1", "-j", "CAMALEON"])
            .output()
            .map_err(|e| {
                SkinshiftError::FirewallError(format!("Failed to add jump to CAMALEON chain: {}", e))
            })?;
            
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(SkinshiftError::FirewallError(
                format!("Failed to add jump to CAMALEON chain: {}", error)
            ));
        }
        
        debug!("CAMALEON chain created successfully");
        
        Ok(())
    }
    
    /// Clear CAMALEON-specific rules
    fn clear_camaleon_rules(&self) -> Result<(), SkinshiftError> {
        debug!("Clearing CAMALEON-specific firewall rules");
        
        // Flush the CAMALEON chain
        let output = Command::new("iptables")
            .args(&["-F", "CAMALEON"])
            .output();
            
        match output {
            Ok(output) => {
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to flush CAMALEON chain: {}", error);
                    // Continue anyway
                }
            }
            Err(e) => {
                warn!("Error flushing CAMALEON chain: {}", e);
                // Continue anyway
            }
        }
        
        // Remove the jump to the CAMALEON chain
        let output = Command::new("iptables")
            .args(&["-D", "INPUT", "-j", "CAMALEON"])
            .output();
            
        match output {
            Ok(output) => {
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to remove jump to CAMALEON chain: {}", error);
                    // Continue anyway
                }
            }
            Err(e) => {
                warn!("Error removing jump to CAMALEON chain: {}", e);
                // Continue anyway
            }
        }
        
        // Delete the CAMALEON chain
        let output = Command::new("iptables")
            .args(&["-X", "CAMALEON"])
            .output();
            
        match output {
            Ok(output) => {
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to delete CAMALEON chain: {}", error);
                    // Continue anyway
                }
            }
            Err(e) => {
                warn!("Error deleting CAMALEON chain: {}", e);
                // Continue anyway
            }
        }
        
        debug!("CAMALEON-specific firewall rules cleared");
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_firewall_rule() {
        let rule = FirewallRule::new("test", "tcp", "ACCEPT")
            .with_source("192.168.1.0/24")
            .with_destination_port("80");
        
        assert_eq!(rule.name, "test");
        assert_eq!(rule.protocol, "tcp");
        assert_eq!(rule.action, "ACCEPT");
        assert_eq!(rule.source, Some("192.168.1.0/24".to_string()));
        assert_eq!(rule.destination_port, Some("80".to_string()));
    }
    
    #[test]
    fn test_to_iptables_args() {
        let rule = FirewallRule::new("test", "tcp", "ACCEPT")
            .with_source("192.168.1.0/24")
            .with_destination_port("80");
        
        let args = rule.to_iptables_args();
        
        assert!(args.contains(&"-p".to_string()));
        assert!(args.contains(&"tcp".to_string()));
        assert!(args.contains(&"-s".to_string()));
        assert!(args.contains(&"192.168.1.0/24".to_string()));
        assert!(args.contains(&"--dport".to_string()));
        assert!(args.contains(&"80".to_string()));
        assert!(args.contains(&"-j".to_string()));
        assert!(args.contains(&"ACCEPT".to_string()));
    }
}
