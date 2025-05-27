use chame_core::adaptive::{AdaptiveError, AdaptiveEvent, AdaptiveHandler};
use chame_core::events::{Event, EventType};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Handler for NetTongue events
pub struct NetTongueHandler {
    /// Event sender
    event_sender: mpsc::Sender<Event>,
}

impl NetTongueHandler {
    /// Create a new NetTongue handler
    pub fn new(event_sender: mpsc::Sender<Event>) -> Self {
        Self { event_sender }
    }
}

#[async_trait]
impl AdaptiveHandler for NetTongueHandler {
    async fn handle_event(&mut self, event: &AdaptiveEvent) -> Result<(), AdaptiveError> {
        // Only handle events that are relevant to network monitoring
        if event.event_type == "NetworkActivity" || event.event_type == "FingerprintChange" {
            // Convert to a network monitoring event
            let network_event = Event::new(
                EventType::NetworkActivity,
                "nettongue_handler",
                Some(serde_json::json!({
                    "original_source": event.source,
                    "severity": event.severity,
                    "details": event.data,
                })),
            );
            
            // Send the event
            if let Err(e) = self.event_sender.send(network_event).await {
                return Err(AdaptiveError::ProcessingFailed(format!(
                    "Failed to send network event: {}",
                    e
                )));
            }
        }
        
        Ok(())
    }
}
