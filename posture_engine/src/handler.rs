use chame_core::adaptive::{AdaptiveError, AdaptiveEvent, AdaptiveHandler};
use chame_core::events::{Event, EventType};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Handler for PostureEngine events
pub struct PostureEngineHandler {
    /// Event sender
    event_sender: mpsc::Sender<Event>,
}

impl PostureEngineHandler {
    /// Create a new PostureEngine handler
    pub fn new(event_sender: mpsc::Sender<Event>) -> Self {
        Self { event_sender }
    }
}

#[async_trait]
impl AdaptiveHandler for PostureEngineHandler {
    async fn handle_event(&mut self, event: &AdaptiveEvent) -> Result<(), AdaptiveError> {
        // Only handle events that are relevant to posture changes
        if event.event_type == "SecurityAlert" || event.event_type == "SystemChange" || 
           event.event_type == "NetworkActivity" {
            // Convert to a posture event
            let posture_event = Event::new(
                EventType::PostureChange,
                "posture_engine_handler",
                Some(serde_json::json!({
                    "original_source": event.source,
                    "severity": event.severity,
                    "details": event.data,
                    "recommendation": "evaluate_posture_change",
                })),
            );
            
            // Send the event
            if let Err(e) = self.event_sender.send(posture_event).await {
                return Err(AdaptiveError::ProcessingFailed(format!(
                    "Failed to send posture event: {}",
                    e
                )));
            }
        }
        
        Ok(())
    }
}
