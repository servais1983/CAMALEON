#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_adaptive_engine_creation() {
        let engine = AdaptiveEngine::new();
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_adaptive_engine_register_handler() {
        let mut engine = AdaptiveEngine::new().unwrap();
        let handler = Arc::new(Mutex::new(MockAdaptiveHandler::new()));
        
        engine.register_handler("test", handler.clone()).await;
        
        let handlers = engine.handlers.read().await;
        assert!(handlers.contains_key("test"));
    }

    #[tokio::test]
    async fn test_adaptive_engine_process_event() {
        let mut engine = AdaptiveEngine::new().unwrap();
        let handler = Arc::new(Mutex::new(MockAdaptiveHandler::new()));
        
        engine.register_handler("test", handler.clone()).await;
        
        let event = AdaptiveEvent {
            source: "test".to_string(),
            event_type: "test_event".to_string(),
            severity: 5,
            data: serde_json::json!({"test": "data"}),
            timestamp: chrono::Utc::now(),
        };
        
        let result = engine.process_event(event).await;
        assert!(result.is_ok());
        
        let handler_lock = handler.lock().await;
        assert_eq!(handler_lock.events_processed, 1);
    }

    struct MockAdaptiveHandler {
        events_processed: usize,
    }

    impl MockAdaptiveHandler {
        fn new() -> Self {
            Self { events_processed: 0 }
        }
    }

    #[async_trait::async_trait]
    impl AdaptiveHandler for MockAdaptiveHandler {
        async fn handle_event(&mut self, _event: &AdaptiveEvent) -> Result<(), AdaptiveError> {
            self.events_processed += 1;
            Ok(())
        }
    }
}
