use crate::errors::ChameleonError;
use crate::events::Event;
use crate::state::ChameleonState;
use crate::Posture;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Adaptive behavior manager that evaluates and changes system posture
pub struct AdaptiveManager {
    /// Reference to the shared system state
    state: Arc<RwLock<ChameleonState>>,
    
    /// Minimum change threshold for posture adjustment
    threshold: f64,
}

impl AdaptiveManager {
    /// Create a new adaptive manager
    pub fn new(state: Arc<RwLock<ChameleonState>>) -> Self {
        Self {
            state,
            threshold: 0.75, // Default threshold (can be configured)
        }
    }
    
    /// Set the threshold for posture changes
    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold.max(0.0).min(1.0);
    }
    
    /// Get the current threshold
    pub fn threshold(&self) -> f64 {
        self.threshold
    }
    
    /// Evaluate whether a posture change is needed based on an event
    pub async fn evaluate_posture_change(&self, event: &Event) -> Result<bool, ChameleonError> {
        debug!("Evaluating posture change based on event: {:?}", event);
        
        // Get current state
        let mut state = self.state.write().await;
        let current_posture = state.current_posture;
        let threat_level = state.threat_level;
        
        // Adjust threat level based on event
        match event.severity() {
            crate::events::Severity::Critical => state.increase_threat_level(0.4),
            crate::events::Severity::High => state.increase_threat_level(0.2),
            crate::events::Severity::Medium => state.increase_threat_level(0.1),
            crate::events::Severity::Low => state.increase_threat_level(0.05),
            crate::events::Severity::Info => {}
        }
        
        // Evaluate if posture change is needed
        let new_posture = self.determine_optimal_posture(state.threat_level);
        
        if new_posture != current_posture {
            if self.should_change_posture(current_posture, new_posture, threat_level, state.threat_level) {
                info!(
                    "Changing posture from {:?} to {:?} (threat level: {:.2} -> {:.2})",
                    current_posture, new_posture, threat_level, state.threat_level
                );
                
                state.current_posture = new_posture;
                state.last_posture_change = Some(chrono::Utc::now());
                
                drop(state); // Release the lock before returning
                return Ok(true);
            }
        }
        
        drop(state); // Release the lock
        Ok(false)
    }
    
    /// Determine the optimal posture based on the current threat level
    fn determine_optimal_posture(&self, threat_level: f64) -> Posture {
        match threat_level {
            t if t < 0.2 => Posture::Neutral,
            t if t < 0.4 => Posture::Silent,
            t if t < 0.6 => Posture::Mimetic,
            t if t < 0.8 => Posture::Fulgurant,
            _ => Posture::Unstable,
        }
    }
    
    /// Decide whether a posture change should occur
    fn should_change_posture(
        &self,
        current: Posture,
        proposed: Posture,
        old_threat: f64,
        new_threat: f64,
    ) -> bool {
        // If the threat difference is significant, change regardless
        if (new_threat - old_threat).abs() > self.threshold {
            return true;
        }
        
        // Check if we're escalating or de-escalating
        let is_escalation = match (current, proposed) {
            (Posture::Neutral, Posture::Silent) => true,
            (Posture::Neutral, Posture::Mimetic) => true,
            (Posture::Neutral, Posture::Fulgurant) => true,
            (Posture::Neutral, Posture::Unstable) => true,
            
            (Posture::Silent, Posture::Mimetic) => true,
            (Posture::Silent, Posture::Fulgurant) => true,
            (Posture::Silent, Posture::Unstable) => true,
            
            (Posture::Mimetic, Posture::Fulgurant) => true,
            (Posture::Mimetic, Posture::Unstable) => true,
            
            (Posture::Fulgurant, Posture::Unstable) => true,
            
            // De-escalation cases
            (Posture::Silent, Posture::Neutral) => false,
            (Posture::Mimetic, Posture::Neutral) => false,
            (Posture::Mimetic, Posture::Silent) => false,
            (Posture::Fulgurant, Posture::Neutral) => false,
            (Posture::Fulgurant, Posture::Silent) => false,
            (Posture::Fulgurant, Posture::Mimetic) => false,
            (Posture::Unstable, Posture::Neutral) => false,
            (Posture::Unstable, Posture::Silent) => false,
            (Posture::Unstable, Posture::Mimetic) => false,
            (Posture::Unstable, Posture::Fulgurant) => false,
            
            // Same posture (should never happen in this function)
            _ => {
                warn!("Unexpected posture comparison: {:?} vs {:?}", current, proposed);
                false
            }
        };
        
        // Be quicker to escalate than de-escalate
        if is_escalation {
            // Escalate more readily
            new_threat > old_threat && new_threat > 0.3
        } else {
            // De-escalate more cautiously
            new_threat < old_threat && (old_threat - new_threat) > 0.2
        }
    }
    
    /// Manually force a specific posture
    pub async fn force_posture(&self, posture: Posture) -> Result<(), ChameleonError> {
        let mut state = self.state.write().await;
        info!(
            "Manually forcing posture change from {:?} to {:?}",
            state.current_posture, posture
        );
        
        state.current_posture = posture;
        state.last_posture_change = Some(chrono::Utc::now());
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Event, EventType};
    
    #[tokio::test]
    async fn test_threat_escalation() {
        let state = Arc::new(RwLock::new(ChameleonState::new()));
        let manager = AdaptiveManager::new(state.clone());
        
        // Initial state
        {
            let mut state = state.write().await;
            state.current_posture = Posture::Neutral;
            state.threat_level = 0.1;
        }
        
        // Create a high severity event
        let event = Event::security_alert(
            "test",
            Some(serde_json::json!({"alert": "High severity test"})),
        );
        
        // Evaluate
        let changed = manager.evaluate_posture_change(&event).await.unwrap();
        
        // Check that the posture has changed and threat level increased
        let state = state.read().await;
        assert!(changed);
        assert_eq!(state.current_posture, Posture::Silent);
        assert!(state.threat_level > 0.1);
    }
}
