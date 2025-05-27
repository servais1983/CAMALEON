#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_event_creation() {
        let event = AdaptiveEvent {
            source: "test_source".to_string(),
            event_type: "test_event".to_string(),
            severity: 3,
            data: serde_json::json!({"key": "value"}),
            timestamp: Utc::now(),
        };
        
        assert_eq!(event.source, "test_source");
        assert_eq!(event.event_type, "test_event");
        assert_eq!(event.severity, 3);
        assert_eq!(event.data["key"], "value");
    }

    #[test]
    fn test_event_serialization() {
        let event = AdaptiveEvent {
            source: "test_source".to_string(),
            event_type: "test_event".to_string(),
            severity: 3,
            data: serde_json::json!({"key": "value"}),
            timestamp: Utc::now(),
        };
        
        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("test_source"));
        assert!(serialized.contains("test_event"));
        assert!(serialized.contains("key"));
        assert!(serialized.contains("value"));
    }

    #[test]
    fn test_event_deserialization() {
        let now = Utc::now();
        let json = format!(
            r#"{{
                "source": "test_source",
                "event_type": "test_event",
                "severity": 3,
                "data": {{"key": "value"}},
                "timestamp": "{}"
            }}"#,
            now.to_rfc3339()
        );
        
        let event: AdaptiveEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.source, "test_source");
        assert_eq!(event.event_type, "test_event");
        assert_eq!(event.severity, 3);
        assert_eq!(event.data["key"], "value");
    }
}
