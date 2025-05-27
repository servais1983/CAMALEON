use chame_core::events::{Event, EventType};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Errors that can occur in the PostureEngine module
#[derive(Error, Debug)]
pub enum PostureEngineError {
    #[error("Invalid posture: {0}")]
    InvalidPosture(String),
    
    #[error("Posture change error: {0}")]
    PostureChange(String),
    
    #[error("Service rotation error: {0}")]
    ServiceRotation(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Defensive postures that the system can adopt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Posture {
    /// Silent mode - minimal visibility
    Silent,
    
    /// Neutral mode - standard server appearance
    Neutral,
    
    /// Mimetic mode - appears as a vulnerable system
    Mimetic,
    
    /// Fulgurant mode - actively disrupts scanning
    Fulgurant,
    
    /// Unstable mode - appears to be malfunctioning
    Unstable,
}

impl Posture {
    /// Convert a string to a posture
    pub fn from_str(s: &str) -> Result<Self, PostureEngineError> {
        match s.to_lowercase().as_str() {
            "silent" => Ok(Self::Silent),
            "neutral" => Ok(Self::Neutral),
            "mimetic" => Ok(Self::Mimetic),
            "fulgurant" => Ok(Self::Fulgurant),
            "unstable" => Ok(Self::Unstable),
            _ => Err(PostureEngineError::InvalidPosture(format!(
                "Unknown posture: {}",
                s
            ))),
        }
    }
    
    /// Convert a posture to a string
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::Neutral => "neutral",
            Self::Mimetic => "mimetic",
            Self::Fulgurant => "fulgurant",
            Self::Unstable => "unstable",
        }
    }
}

/// Configuration for the PostureEngine module
#[derive(Debug, Clone)]
pub struct PostureEngineConfig {
    /// Confidence threshold to trigger posture change
    pub change_threshold: f64,
    
    /// Whether service rotation is enabled
    pub service_rotation_enabled: bool,
    
    /// Service rotation interval in seconds
    pub service_rotation_interval: u64,
    
    /// Available postures
    pub postures: Vec<Posture>,
}

impl Default for PostureEngineConfig {
    fn default() -> Self {
        Self {
            change_threshold: 0.75,
            service_rotation_enabled: false,
            service_rotation_interval: 7200,
            postures: vec![
                Posture::Silent,
                Posture::Neutral,
                Posture::Mimetic,
                Posture::Fulgurant,
                Posture::Unstable,
            ],
        }
    }
}

/// Main PostureEngine service
pub struct PostureEngine {
    /// Configuration
    config: PostureEngineConfig,
    
    /// Current posture
    current_posture: RwLock<Posture>,
    
    /// Posture history
    posture_history: RwLock<Vec<(Posture, chrono::DateTime<chrono::Utc>)>>,
    
    /// Event sender
    event_sender: tokio::sync::mpsc::Sender<Event>,
    
    /// Service rotator
    service_rotator: Option<Arc<ServiceRotator>>,
}

impl PostureEngine {
    /// Create a new PostureEngine instance
    pub async fn new(
        config: PostureEngineConfig,
        event_sender: tokio::sync::mpsc::Sender<Event>,
    ) -> Result<Self, PostureEngineError> {
        let service_rotator = if config.service_rotation_enabled {
            Some(Arc::new(ServiceRotator::new(config.service_rotation_interval)))
        } else {
            None
        };
        
        Ok(Self {
            config,
            current_posture: RwLock::new(Posture::Neutral),
            posture_history: RwLock::new(Vec::new()),
            event_sender,
            service_rotator,
        })
    }
    
    /// Start the posture engine
    pub async fn start(&self) -> Result<(), PostureEngineError> {
        tracing::info!("Starting PostureEngine");
        
        // Start service rotation if enabled
        if let Some(rotator) = &self.service_rotator {
            rotator.start().await?;
        }
        
        Ok(())
    }
    
    /// Stop the posture engine
    pub async fn stop(&self) -> Result<(), PostureEngineError> {
        tracing::info!("Stopping PostureEngine");
        
        // Stop service rotation if enabled
        if let Some(rotator) = &self.service_rotator {
            rotator.stop().await?;
        }
        
        Ok(())
    }
    
    /// Get the current posture
    pub async fn get_current_posture(&self) -> Posture {
        let posture = self.current_posture.read().await;
        posture.clone()
    }
    
    /// Set the current posture
    pub async fn set_posture(&self, posture: Posture) -> Result<(), PostureEngineError> {
        // Check if the posture is valid
        if !self.config.postures.contains(&posture) {
            return Err(PostureEngineError::InvalidPosture(format!(
                "Posture not in allowed list: {:?}",
                posture
            )));
        }
        
        // Update the current posture
        {
            let mut current = self.current_posture.write().await;
            *current = posture.clone();
        }
        
        // Add to history
        {
            let mut history = self.posture_history.write().await;
            history.push((posture.clone(), chrono::Utc::now()));
        }
        
        // Send event
        let event = Event::posture_change(
            "posture_engine",
            Some(serde_json::json!({
                "posture": posture.to_str(),
                "timestamp": chrono::Utc::now(),
            })),
        );
        
        if let Err(e) = self.event_sender.send(event).await {
            tracing::error!("Failed to send posture change event: {}", e);
        }
        
        tracing::info!("Posture changed to: {:?}", posture);
        
        Ok(())
    }
    
    /// Evaluate events and potentially change posture
    pub async fn evaluate_events(&self, events: &[Event]) -> Result<bool, PostureEngineError> {
        // Count security-related events by severity
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        
        for event in events {
            match event.severity() {
                chame_core::events::Severity::Critical => critical_count += 1,
                chame_core::events::Severity::High => high_count += 1,
                chame_core::events::Severity::Medium => medium_count += 1,
                _ => {}
            }
        }
        
        // Calculate threat level (0.0 - 1.0)
        let threat_level = if events.is_empty() {
            0.0
        } else {
            let weighted_sum = critical_count as f64 * 1.0 + high_count as f64 * 0.7 + medium_count as f64 * 0.3;
            let max_possible = events.len() as f64;
            (weighted_sum / max_possible).min(1.0)
        };
        
        // Determine if posture change is needed
        if threat_level >= self.config.change_threshold {
            let current_posture = self.get_current_posture().await;
            let new_posture = self.determine_best_posture(threat_level, events).await;
            
            if current_posture != new_posture {
                self.set_posture(new_posture).await?;
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Determine the best posture based on threat level and events
    async fn determine_best_posture(&self, threat_level: f64, events: &[Event]) -> Posture {
        if threat_level >= 0.9 {
            // High threat, use fulgurant or unstable
            if events.iter().any(|e| matches!(e.event_type, EventType::SecurityAlert)) {
                Posture::Fulgurant
            } else {
                Posture::Unstable
            }
        } else if threat_level >= 0.6 {
            // Medium-high threat, use mimetic
            Posture::Mimetic
        } else if threat_level >= 0.3 {
            // Medium threat, use neutral
            Posture::Neutral
        } else {
            // Low threat, use silent
            Posture::Silent
        }
    }
    
    /// Get posture history
    pub async fn get_posture_history(&self) -> Vec<(Posture, chrono::DateTime<chrono::Utc>)> {
        let history = self.posture_history.read().await;
        history.clone()
    }
}

/// Service rotator for changing exposed services
pub struct ServiceRotator {
    /// Rotation interval in seconds
    interval: u64,
    
    /// Whether the rotator is running
    running: RwLock<bool>,
}

impl ServiceRotator {
    /// Create a new service rotator
    pub fn new(interval: u64) -> Self {
        Self {
            interval,
            running: RwLock::new(false),
        }
    }
    
    /// Start service rotation
    pub async fn start(&self) -> Result<(), PostureEngineError> {
        let mut running = self.running.write().await;
        *running = true;
        tracing::info!("Service rotation started (interval: {} seconds)", self.interval);
        Ok(())
    }
    
    /// Stop service rotation
    pub async fn stop(&self) -> Result<(), PostureEngineError> {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Service rotation stopped");
        Ok(())
    }
}
