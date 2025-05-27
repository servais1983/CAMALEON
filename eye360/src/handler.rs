use chame_core::adaptive::{AdaptiveError, AdaptiveEvent, AdaptiveHandler};
use chame_core::events::{Event, EventType};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Handler for Eye360 events
pub struct Eye360Handler {
    /// Event sender
    event_sender: mpsc::Sender<Event>,
}

impl Eye360Handler {
    /// Create a new Eye360 handler
    pub fn new(event_sender: mpsc::Sender<Event>) -> Self {
        Self { event_sender }
    }
}

#[async_trait]
impl AdaptiveHandler for Eye360Handler {
    async fn handle_event(&mut self, event: &AdaptiveEvent) -> Result<(), AdaptiveError> {
        // Only handle events that are relevant to system monitoring
        if event.event_type == "SecurityAlert" || event.event_type == "SystemChange" {
            // Convert to a system monitoring event
            let system_event = Event::new(
                EventType::SystemChange,
                "eye360_handler",
                Some(serde_json::json!({
                    "original_source": event.source,
                    "severity": event.severity,
                    "details": event.data,
                })),
            );
            
            // Send the event
            if let Err(e) = self.event_sender.send(system_event).await {
                return Err(AdaptiveError::ProcessingFailed(format!(
                    "Failed to send system event: {}",
                    e
                )));
            }
        }
        
        Ok(())
    }
}
