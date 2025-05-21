use crate::Posture;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service status states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Service has been created but not initialized
    Created,
    
    /// Service is initializing
    Initializing,
    
    /// Service is ready but not running
    Ready,
    
    /// Service is starting
    Starting,
    
    /// Service is running
    Running,
    
    /// Service is paused
    Paused,
    
    /// Service is stopping
    Stopping,
    
    /// Service is stopped
    Stopped,
    
    /// Service has encountered an error
    Error,
    
    /// Service is in an unknown state
    Unknown,
}

/// Internal state of the CAMALEON system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChameleonState {
    /// Current status of the service
    pub status: Status,
    
    /// Current defensive posture
    pub current_posture: Posture,
    
    /// When the service was started
    pub started_at: Option<DateTime<Utc>>,
    
    /// When the posture was last changed
    pub last_posture_change: Option<DateTime<Utc>>,
    
    /// Threat level assessment (0.0 - 1.0)
    pub threat_level: f64,
    
    /// Active services
    pub active_services: HashMap<String, ServiceInfo>,
    
    /// Deployed honeypots
    pub active_honeypots: Vec<HoneypotInfo>,
    
    /// Current system fingerprint
    pub current_fingerprint: Option<FingerprintInfo>,
}

impl ChameleonState {
    /// Create a new state with default values
    pub fn new() -> Self {
        Self {
            status: Status::Created,
            current_posture: Posture::Neutral,
            started_at: None,
            last_posture_change: None,
            threat_level: 0.0,
            active_services: HashMap::new(),
            active_honeypots: Vec::new(),
            current_fingerprint: None,
        }
    }
    
    /// Get a copy of the current system state (for external use)
    pub fn get_system_state(&self) -> SystemState {
        SystemState {
            status: self.status,
            current_posture: self.current_posture,
            started_at: self.started_at,
            last_posture_change: self.last_posture_change,
            threat_level: self.threat_level,
            active_services_count: self.active_services.len(),
            active_honeypots_count: self.active_honeypots.len(),
            current_fingerprint: self.current_fingerprint.clone(),
        }
    }
    
    /// Increase the threat level (capped at 1.0)
    pub fn increase_threat_level(&mut self, amount: f64) {
        self.threat_level = (self.threat_level + amount).min(1.0);
    }
    
    /// Decrease the threat level (floored at 0.0)
    pub fn decrease_threat_level(&mut self, amount: f64) {
        self.threat_level = (self.threat_level - amount).max(0.0);
    }
    
    /// Add a service to active services
    pub fn add_service(&mut self, name: String, info: ServiceInfo) {
        self.active_services.insert(name, info);
    }
    
    /// Remove a service from active services
    pub fn remove_service(&mut self, name: &str) -> Option<ServiceInfo> {
        self.active_services.remove(name)
    }
    
    /// Add a honeypot
    pub fn add_honeypot(&mut self, info: HoneypotInfo) {
        self.active_honeypots.push(info);
    }
    
    /// Remove a honeypot by ID
    pub fn remove_honeypot(&mut self, id: &str) -> Option<HoneypotInfo> {
        if let Some(index) = self.active_honeypots.iter().position(|h| h.id == id) {
            Some(self.active_honeypots.remove(index))
        } else {
            None
        }
    }
    
    /// Set the current fingerprint
    pub fn set_fingerprint(&mut self, fingerprint: FingerprintInfo) {
        self.current_fingerprint = Some(fingerprint);
    }
    
    /// Clear the current fingerprint
    pub fn clear_fingerprint(&mut self) {
        self.current_fingerprint = None;
    }
}

impl Default for ChameleonState {
    fn default() -> Self {
        Self::new()
    }
}

/// External view of the system state (safe to share)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    /// Current status of the service
    pub status: Status,
    
    /// Current defensive posture
    pub current_posture: Posture,
    
    /// When the service was started
    pub started_at: Option<DateTime<Utc>>,
    
    /// When the posture was last changed
    pub last_posture_change: Option<DateTime<Utc>>,
    
    /// Threat level assessment (0.0 - 1.0)
    pub threat_level: f64,
    
    /// Number of active services
    pub active_services_count: usize,
    
    /// Number of active honeypots
    pub active_honeypots_count: usize,
    
    /// Current system fingerprint
    pub current_fingerprint: Option<FingerprintInfo>,
}

/// Information about a running service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    
    /// Service type
    pub service_type: String,
    
    /// Port the service is running on
    pub port: u16,
    
    /// When the service was started
    pub started_at: DateTime<Utc>,
    
    /// Configuration properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Information about a deployed honeypot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoneypotInfo {
    /// Unique ID
    pub id: String,
    
    /// Type of honeypot
    pub honeypot_type: String,
    
    /// Port the honeypot is running on
    pub port: u16,
    
    /// When the honeypot was deployed
    pub deployed_at: DateTime<Utc>,
    
    /// Configuration properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Information about the current system fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintInfo {
    /// Name of the fingerprint (e.g., "win2008_smb1")
    pub name: String,
    
    /// Operating system family
    pub os_family: String,
    
    /// Operating system version
    pub os_version: Option<String>,
    
    /// TTL value
    pub ttl: Option<u8>,
    
    /// MSS value
    pub mss: Option<u16>,
    
    /// Window size
    pub window_size: Option<u32>,
    
    /// When the fingerprint was applied
    pub applied_at: DateTime<Utc>,
    
    /// Additional properties
    pub properties: HashMap<String, serde_json::Value>,
}
