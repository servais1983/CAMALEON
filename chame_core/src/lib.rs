mod adaptive;
mod errors;
mod events;
mod metrics;
mod state;

use adaptive::AdaptiveManager;
use errors::ChameleonError;
use events::{Event, EventType};
use metrics::MetricsCollector;
use state::{ChameleonState, SystemState};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Current posture of the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Posture {
    /// No visible services, minimal footprint
    Silent,
    /// Standard server appearance
    Neutral,
    /// Mimicking a specific target system
    Mimetic,
    /// Aggressive counter-measures
    Fulgurant,
    /// Unpredictable behavior to confuse attackers
    Unstable,
}

impl std::fmt::Display for Posture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Posture::Silent => write!(f, "Silent"),
            Posture::Neutral => write!(f, "Neutral"),
            Posture::Mimetic => write!(f, "Mimetic"),
            Posture::Fulgurant => write!(f, "Fulgurant"),
            Posture::Unstable => write!(f, "Unstable"),
        }
    }
}

impl Posture {
    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "silent" => Some(Posture::Silent),
            "neutral" => Some(Posture::Neutral),
            "mimetic" => Some(Posture::Mimetic),
            "fulgurant" => Some(Posture::Fulgurant),
            "unstable" => Some(Posture::Unstable),
            _ => None,
        }
    }
}

/// Core service trait that all CAMALEON components must implement
#[async_trait]
pub trait ChameleonService: Send + Sync {
    /// Initialize the service
    async fn init(&self) -> Result<(), ChameleonError>;
    
    /// Start the service
    async fn start(&self) -> Result<(), ChameleonError>;
    
    /// Stop the service
    async fn stop(&self) -> Result<(), ChameleonError>;
    
    /// Handle an event
    async fn handle_event(&self, event: Event) -> Result<(), ChameleonError>;
    
    /// Change the posture of the service
    async fn change_posture(&self, posture: Posture) -> Result<(), ChameleonError>;
    
    /// Get the current state
    async fn get_state(&self) -> Result<SystemState, ChameleonError>;
}

/// Main CAMALEON core service
#[derive(Clone)]
pub struct ChameleonCore {
    state: Arc<RwLock<ChameleonState>>,
    adaptive_manager: Arc<AdaptiveManager>,
    metrics: Arc<MetricsCollector>,
}

impl ChameleonCore {
    /// Create a new ChameleonCore instance
    pub fn new() -> Self {
        let state = Arc::new(RwLock::new(ChameleonState::new()));
        let adaptive_manager = Arc::new(AdaptiveManager::new(state.clone()));
        let metrics = Arc::new(MetricsCollector::new());
        
        Self {
            state,
            adaptive_manager,
            metrics,
        }
    }
    
    /// Register a new event
    pub async fn register_event(&self, event_type: EventType, source: &str, data: Option<serde_json::Value>) -> Result<(), ChameleonError> {
        let event = Event {
            timestamp: Utc::now(),
            event_type,
            source: source.to_string(),
            data,
        };
        
        self.handle_event(event).await
    }
    
    /// Get metrics within a time range
    pub async fn get_metrics(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<serde_json::Value, ChameleonError> {
        self.metrics.get_metrics(start, end).await
    }
}

#[async_trait]
impl ChameleonService for ChameleonCore {
    async fn init(&self) -> Result<(), ChameleonError> {
        info!("Initializing CAMALEON core service");
        let mut state = self.state.write().await;
        state.status = state::Status::Initializing;
        // Initial setup code would go here
        state.status = state::Status::Ready;
        Ok(())
    }
    
    async fn start(&self) -> Result<(), ChameleonError> {
        info!("Starting CAMALEON core service");
        let mut state = self.state.write().await;
        state.status = state::Status::Starting;
        // Startup code would go here
        state.status = state::Status::Running;
        Ok(())
    }
    
    async fn stop(&self) -> Result<(), ChameleonError> {
        info!("Stopping CAMALEON core service");
        let mut state = self.state.write().await;
        state.status = state::Status::Stopping;
        // Cleanup code would go here
        state.status = state::Status::Stopped;
        Ok(())
    }
    
    async fn handle_event(&self, event: Event) -> Result<(), ChameleonError> {
        debug!("Handling event: {:?}", event);
        
        // Record the event
        self.metrics.record_event(&event).await?;
        
        // Process event based on type
        match event.event_type {
            EventType::SecurityAlert => {
                warn!("Security alert detected: {:?}", event);
                self.adaptive_manager.evaluate_posture_change(&event).await?;
            },
            EventType::SystemChange => {
                info!("System change detected: {:?}", event);
                // Handle system change
            },
            EventType::NetworkActivity => {
                debug!("Network activity detected: {:?}", event);
                // Handle network activity
            },
            EventType::PostureChange => {
                info!("Posture change event: {:?}", event);
                // Handle posture change
            },
            _ => {
                debug!("Other event type: {:?}", event);
                // Handle other event types
            }
        }
        
        Ok(())
    }
    
    async fn change_posture(&self, posture: Posture) -> Result<(), ChameleonError> {
        info!("Changing posture to: {}", posture);
        
        let mut state = self.state.write().await;
        let old_posture = state.current_posture;
        state.current_posture = posture;
        
        // Register the posture change event
        drop(state); // Release the lock before calling register_event
        
        self.register_event(
            EventType::PostureChange,
            "core",
            Some(serde_json::json!({
                "old_posture": old_posture.to_string(),
                "new_posture": posture.to_string(),
            })),
        ).await?;
        
        Ok(())
    }
    
    async fn get_state(&self) -> Result<SystemState, ChameleonError> {
        let state = self.state.read().await;
        Ok(state.get_system_state())
    }
}

impl Default for ChameleonCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_core() {
        let core = ChameleonCore::new();
        assert!(matches!(core.state.read().await.status, state::Status::Created));
    }
    
    #[tokio::test]
    async fn test_change_posture() {
        let core = ChameleonCore::new();
        
        // Check initial posture
        {
            let state = core.state.read().await;
            assert_eq!(state.current_posture, Posture::Neutral);
        }
        
        // Change posture
        core.change_posture(Posture::Silent).await.unwrap();
        
        // Check new posture
        {
            let state = core.state.read().await;
            assert_eq!(state.current_posture, Posture::Silent);
        }
    }
}
