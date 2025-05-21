use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of events that the system can handle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Security-related alerts and warnings
    SecurityAlert,
    
    /// System state or configuration changes
    SystemChange,
    
    /// Network activity detection
    NetworkActivity,
    
    /// Defensive posture changes
    PostureChange,
    
    /// Honeypot activity
    HoneypotActivity,
    
    /// Fingerprint changes
    FingerprintChange,
    
    /// Service start/stop events
    ServiceLifecycle,
    
    /// Internal metrics and health checks
    MetricsReport,
    
    /// Custom event types
    Custom(String),
}

/// An event in the CAMALEON system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    
    /// Type of event
    pub event_type: EventType,
    
    /// Source component that generated the event
    pub source: String,
    
    /// Additional data payload (JSON format)
    pub data: Option<serde_json::Value>,
}

impl Event {
    /// Create a new event
    pub fn new(
        event_type: EventType,
        source: impl Into<String>,
        data: Option<serde_json::Value>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type,
            source: source.into(),
            data,
        }
    }
    
    /// Create a security alert event
    pub fn security_alert(source: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self::new(EventType::SecurityAlert, source, data)
    }
    
    /// Create a system change event
    pub fn system_change(source: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self::new(EventType::SystemChange, source, data)
    }
    
    /// Create a network activity event
    pub fn network_activity(source: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self::new(EventType::NetworkActivity, source, data)
    }
    
    /// Create a posture change event
    pub fn posture_change(source: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self::new(EventType::PostureChange, source, data)
    }
    
    /// Create a honeypot activity event
    pub fn honeypot_activity(source: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self::new(EventType::HoneypotActivity, source, data)
    }
    
    /// Create a fingerprint change event
    pub fn fingerprint_change(source: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self::new(EventType::FingerprintChange, source, data)
    }
    
    /// Create a service lifecycle event
    pub fn service_lifecycle(source: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self::new(EventType::ServiceLifecycle, source, data)
    }
    
    /// Create a metrics report event
    pub fn metrics_report(source: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self::new(EventType::MetricsReport, source, data)
    }
    
    /// Create a custom event
    pub fn custom(custom_type: impl Into<String>, source: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self::new(EventType::Custom(custom_type.into()), source, data)
    }
    
    /// Get the event severity (derived from event type)
    pub fn severity(&self) -> Severity {
        match self.event_type {
            EventType::SecurityAlert => Severity::High,
            EventType::PostureChange => Severity::Medium,
            EventType::SystemChange => Severity::Medium,
            EventType::HoneypotActivity => Severity::Medium,
            EventType::FingerprintChange => Severity::Low,
            EventType::ServiceLifecycle => Severity::Low,
            EventType::NetworkActivity => Severity::Info,
            EventType::MetricsReport => Severity::Info,
            EventType::Custom(_) => Severity::Info,
        }
    }
}

/// Severity levels for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Critical severity - needs immediate attention
    Critical,
    
    /// High severity - needs urgent attention
    High,
    
    /// Medium severity - needs attention soon
    Medium,
    
    /// Low severity - routine events
    Low,
    
    /// Informational - no action needed
    Info,
}
