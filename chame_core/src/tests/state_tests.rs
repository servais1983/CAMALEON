#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_manager_creation() {
        let manager = StateManager::new();
        assert!(manager.current_state.read().unwrap().is_empty());
    }
    
    #[test]
    fn test_state_manager_set_get() {
        let manager = StateManager::new();
        
        // Set a string value
        manager.set("test_key", "test_value");
        
        // Get the value
        let value: Option<String> = manager.get("test_key");
        assert_eq!(value, Some("test_value".to_string()));
    }
    
    #[test]
    fn test_state_manager_set_get_complex() {
        let manager = StateManager::new();
        
        // Create a complex structure
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct TestStruct {
            field1: String,
            field2: i32,
            field3: bool,
        }
        
        let test_struct = TestStruct {
            field1: "hello".to_string(),
            field2: 42,
            field3: true,
        };
        
        // Set the complex value
        manager.set("test_struct", test_struct.clone());
        
        // Get the complex value
        let value: Option<TestStruct> = manager.get("test_struct");
        assert_eq!(value, Some(test_struct));
    }
    
    #[test]
    fn test_state_manager_remove() {
        let manager = StateManager::new();
        
        // Set a value
        manager.set("test_key", "test_value");
        
        // Verify it exists
        let value: Option<String> = manager.get("test_key");
        assert_eq!(value, Some("test_value".to_string()));
        
        // Remove the value
        manager.remove("test_key");
        
        // Verify it's gone
        let value: Option<String> = manager.get("test_key");
        assert_eq!(value, None);
    }
    
    #[test]
    fn test_state_manager_clear() {
        let manager = StateManager::new();
        
        // Set multiple values
        manager.set("key1", "value1");
        manager.set("key2", "value2");
        manager.set("key3", "value3");
        
        // Verify they exist
        let value1: Option<String> = manager.get("key1");
        let value2: Option<String> = manager.get("key2");
        assert_eq!(value1, Some("value1".to_string()));
        assert_eq!(value2, Some("value2".to_string()));
        
        // Clear all state
        manager.clear();
        
        // Verify all values are gone
        let value1: Option<String> = manager.get("key1");
        let value2: Option<String> = manager.get("key2");
        assert_eq!(value1, None);
        assert_eq!(value2, None);
    }
}
