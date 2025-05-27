use crate::events::{Event, EventType, Severity};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};

/// Errors that can occur in the adaptive engine
#[derive(Error, Debug)]
pub enum AdaptiveError {
    #[error("Handler not found: {0}")]
    HandlerNotFound(String),
    
    #[error("Event processing failed: {0}")]
    ProcessingFailed(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// An event that triggers adaptive behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveEvent {
    /// Source of the event
    pub source: String,
    
    /// Type of event
    pub event_type: String,
    
    /// Severity level (0-10)
    pub severity: u8,
    
    /// Additional data
    pub data: serde_json::Value,
    
    /// When the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl From<Event> for AdaptiveEvent {
    fn from(event: Event) -> Self {
        let severity = match event.severity() {
            Severity::Critical => 10,
            Severity::High => 8,
            Severity::Medium => 5,
            Severity::Low => 3,
            Severity::Info => 1,
        };
        
        Self {
            source: event.source,
            event_type: match &event.event_type {
                EventType::Custom(name) => name.clone(),
                other => format!("{:?}", other),
            },
            severity,
            data: event.data.unwrap_or(serde_json::json!({})),
            timestamp: event.timestamp,
        }
    }
}

/// Handler for adaptive events
#[async_trait]
pub trait AdaptiveHandler: Send + Sync {
    /// Handle an adaptive event
    async fn handle_event(&mut self, event: &AdaptiveEvent) -> Result<(), AdaptiveError>;
}

/// Engine that processes adaptive events and triggers appropriate responses
pub struct AdaptiveEngine {
    /// Registered handlers
    pub handlers: RwLock<HashMap<String, Arc<Mutex<dyn AdaptiveHandler>>>>,
    
    /// Event history
    pub history: RwLock<Vec<AdaptiveEvent>>,
    
    /// Maximum history size
    pub max_history: usize,
}

impl AdaptiveEngine {
    /// Create a new adaptive engine
    pub fn new() -> Result<Self, AdaptiveError> {
        Ok(Self {
            handlers: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
            max_history: 1000,
        })
    }
    
    /// Register a handler for adaptive events
    pub async fn register_handler(
        &mut self,
        name: impl Into<String>,
        handler: Arc<Mutex<dyn AdaptiveHandler>>,
    ) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(name.into(), handler);
    }
    
    /// Process an adaptive event
    pub async fn process_event(&self, event: AdaptiveEvent) -> Result<(), AdaptiveError> {
        // Add to history
        {
            let mut history = self.history.write().await;
            history.push(event.clone());
            
            // Trim history if needed
            if history.len() > self.max_history {
                history.remove(0);
            }
        }
        
        // Process with all handlers
        let handlers = self.handlers.read().await;
        for (name, handler) in handlers.iter() {
            let mut handler_lock = handler.lock().await;
            if let Err(e) = handler_lock.handle_event(&event).await {
                return Err(AdaptiveError::ProcessingFailed(format!(
                    "Handler '{}' failed: {}",
                    name, e
                )));
            }
        }
        
        Ok(())
    }
    
    /// Get the event history
    pub async fn get_history(&self) -> Vec<AdaptiveEvent> {
        let history = self.history.read().await;
        history.clone()
    }
    
    /// Clear the event history
    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.clear();
    }
    
    /// Create an adaptive event from components
    pub fn create_event(
        source: impl Into<String>,
        event_type: impl Into<String>,
        severity: u8,
        data: serde_json::Value,
    ) -> AdaptiveEvent {
        AdaptiveEvent {
            source: source.into(),
            event_type: event_type.into(),
            severity,
            data,
            timestamp: Utc::now(),
        }
    }
}
