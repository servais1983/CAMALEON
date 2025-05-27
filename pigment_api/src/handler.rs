use chame_core::adaptive::{AdaptiveError, AdaptiveEvent, AdaptiveHandler};
use chame_core::events::{Event, EventType};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Handler for PigmentAPI events
pub struct PigmentApiHandler {
    /// Event sender
    event_sender: mpsc::Sender<Event>,
}

impl PigmentApiHandler {
    /// Create a new PigmentAPI handler
    pub fn new(event_sender: mpsc::Sender<Event>) -> Self {
        Self { event_sender }
    }
}

#[async_trait]
impl AdaptiveHandler for PigmentApiHandler {
    async fn handle_event(&mut self, event: &AdaptiveEvent) -> Result<(), AdaptiveError> {
        // Handle all events to update the API state
        let api_event = Event::new(
            EventType::MetricsReport,
            "pigment_api_handler",
            Some(serde_json::json!({
                "original_source": event.source,
                "severity": event.severity,
                "details": event.data,
                "timestamp": event.timestamp,
            })),
        );
        
        // Send the event
        if let Err(e) = self.event_sender.send(api_event).await {
            return Err(AdaptiveError::ProcessingFailed(format!(
                "Failed to send API event: {}",
                e
            )));
        }
        
        Ok(())
    }
}
