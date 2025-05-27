#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_counter_creation() {
        let counter = MetricsCounter::new("test_counter");
        assert_eq!(counter.name, "test_counter");
        assert_eq!(counter.value(), 0);
    }
    
    #[test]
    fn test_metrics_counter_increment() {
        let mut counter = MetricsCounter::new("test_counter");
        counter.increment();
        assert_eq!(counter.value(), 1);
        
        counter.increment_by(5);
        assert_eq!(counter.value(), 6);
    }
    
    #[test]
    fn test_metrics_gauge_creation() {
        let gauge = MetricsGauge::new("test_gauge");
        assert_eq!(gauge.name, "test_gauge");
        assert_eq!(gauge.value(), 0.0);
    }
    
    #[test]
    fn test_metrics_gauge_set() {
        let mut gauge = MetricsGauge::new("test_gauge");
        gauge.set(42.5);
        assert_eq!(gauge.value(), 42.5);
    }
    
    #[test]
    fn test_metrics_registry_creation() {
        let registry = MetricsRegistry::new();
        assert!(registry.counters.read().unwrap().is_empty());
        assert!(registry.gauges.read().unwrap().is_empty());
    }
    
    #[test]
    fn test_metrics_registry_register_counter() {
        let registry = MetricsRegistry::new();
        registry.register_counter("test_counter");
        
        let counters = registry.counters.read().unwrap();
        assert!(counters.contains_key("test_counter"));
    }
    
    #[test]
    fn test_metrics_registry_register_gauge() {
        let registry = MetricsRegistry::new();
        registry.register_gauge("test_gauge");
        
        let gauges = registry.gauges.read().unwrap();
        assert!(gauges.contains_key("test_gauge"));
    }
    
    #[test]
    fn test_metrics_registry_increment_counter() {
        let registry = MetricsRegistry::new();
        registry.register_counter("test_counter");
        registry.increment_counter("test_counter");
        
        let counters = registry.counters.read().unwrap();
        let counter = counters.get("test_counter").unwrap();
        assert_eq!(counter.value(), 1);
    }
    
    #[test]
    fn test_metrics_registry_set_gauge() {
        let registry = MetricsRegistry::new();
        registry.register_gauge("test_gauge");
        registry.set_gauge("test_gauge", 42.5);
        
        let gauges = registry.gauges.read().unwrap();
        let gauge = gauges.get("test_gauge").unwrap();
        assert_eq!(gauge.value(), 42.5);
    }
}
