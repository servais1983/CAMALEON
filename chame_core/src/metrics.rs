use crate::errors::ChameleonError;
use crate::events::Event;
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Store and analyze system metrics
pub struct MetricsCollector {
    /// Event history with timestamp index
    events: Arc<RwLock<Vec<Event>>>,
    
    /// Various counters for quick lookups
    counters: Arc<DashMap<String, u64>>,
    
    /// Gauges for continuous measurements
    gauges: Arc<DashMap<String, f64>>,
    
    /// Time series data for trends
    time_series: Arc<DashMap<String, Vec<TimeSeriesPoint>>>,
}

/// A point in a time series
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeSeriesPoint {
    /// Timestamp of the measurement
    timestamp: DateTime<Utc>,
    
    /// Value of the measurement
    value: f64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            counters: Arc::new(DashMap::new()),
            gauges: Arc::new(DashMap::new()),
            time_series: Arc::new(DashMap::new()),
        }
    }
    
    /// Record an event
    pub async fn record_event(&self, event: &Event) -> Result<(), ChameleonError> {
        // Add to event history
        {
            let mut events = self.events.write().await;
            events.push(event.clone());
            
            // Trim event history if it gets too large (keep the most recent 10000 events)
            if events.len() > 10000 {
                *events = events.split_off(events.len() - 10000);
            }
        }
        
        // Update counter for this event type
        let counter_key = format!("event_count_{:?}", event.event_type);
        self.increment_counter(&counter_key);
        
        // Update counter for this source
        let source_key = format!("event_source_{}", event.source);
        self.increment_counter(&source_key);
        
        // Update counter for severity
        let severity_key = format!("severity_{:?}", event.severity());
        self.increment_counter(&severity_key);
        
        Ok(())
    }
    
    /// Increment a counter
    pub fn increment_counter(&self, key: &str) {
        self.counters
            .entry(key.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }
    
    /// Set a counter to a specific value
    pub fn set_counter(&self, key: &str, value: u64) {
        self.counters.insert(key.to_string(), value);
    }
    
    /// Get a counter value
    pub fn get_counter(&self, key: &str) -> u64 {
        self.counters.get(key).map(|v| *v).unwrap_or(0)
    }
    
    /// Set a gauge value
    pub fn set_gauge(&self, key: &str, value: f64) {
        self.gauges.insert(key.to_string(), value);
    }
    
    /// Get a gauge value
    pub fn get_gauge(&self, key: &str) -> Option<f64> {
        self.gauges.get(key).map(|v| *v)
    }
    
    /// Add a time series point
    pub fn add_time_series_point(&self, key: &str, value: f64) {
        let point = TimeSeriesPoint {
            timestamp: Utc::now(),
            value,
        };
        
        self.time_series
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(point);
    }
    
    /// Get metrics within a time range
    pub async fn get_metrics(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<serde_json::Value, ChameleonError> {
        // Filter events in the time range
        let events = {
            let events = self.events.read().await;
            events
                .iter()
                .filter(|e| e.timestamp >= start && e.timestamp <= end)
                .cloned()
                .collect::<Vec<_>>()
        };
        
        // Count events by type
        let mut event_counts = std::collections::HashMap::new();
        for event in &events {
            let key = format!("{:?}", event.event_type);
            *event_counts.entry(key).or_insert(0u64) += 1;
        }
        
        // Count events by source
        let mut source_counts = std::collections::HashMap::new();
        for event in &events {
            *source_counts.entry(event.source.clone()).or_insert(0u64) += 1;
        }
        
        // Count events by severity
        let mut severity_counts = std::collections::HashMap::new();
        for event in &events {
            let key = format!("{:?}", event.severity());
            *severity_counts.entry(key).or_insert(0u64) += 1;
        }
        
        // Get time series data within range
        let mut time_series_data = std::collections::HashMap::new();
        for entry in self.time_series.iter() {
            let key = entry.key().clone();
            let points = entry.value()
                .iter()
                .filter(|p| p.timestamp >= start && p.timestamp <= end)
                .cloned()
                .collect::<Vec<_>>();
            
            if !points.is_empty() {
                let serialized_points = points.iter().map(|p| {
                    serde_json::json!({
                        "timestamp": p.timestamp,
                        "value": p.value,
                    })
                }).collect::<Vec<_>>();
                
                time_series_data.insert(key, serialized_points);
            }
        }
        
        // Build the full metrics report
        let metrics = serde_json::json!({
            "time_range": {
                "start": start,
                "end": end,
                "duration_seconds": (end - start).num_seconds(),
            },
            "event_counts": {
                "total": events.len(),
                "by_type": event_counts,
                "by_source": source_counts,
                "by_severity": severity_counts,
            },
            "counters": self.get_all_counters(),
            "gauges": self.get_all_gauges(),
            "time_series": time_series_data,
        });
        
        Ok(metrics)
    }
    
    /// Get all counters as a HashMap
    fn get_all_counters(&self) -> std::collections::HashMap<String, u64> {
        let mut result = std::collections::HashMap::new();
        for entry in self.counters.iter() {
            result.insert(entry.key().clone(), *entry.value());
        }
        result
    }
    
    /// Get all gauges as a HashMap
    fn get_all_gauges(&self) -> std::collections::HashMap<String, f64> {
        let mut result = std::collections::HashMap::new();
        for entry in self.gauges.iter() {
            result.insert(entry.key().clone(), *entry.value());
        }
        result
    }
    
    /// Calculate the rate of events per second within a time window
    pub async fn calculate_event_rate(&self, window_seconds: i64) -> Result<f64, ChameleonError> {
        let now = Utc::now();
        let window_start = now - Duration::seconds(window_seconds);
        
        let events = {
            let events = self.events.read().await;
            events
                .iter()
                .filter(|e| e.timestamp >= window_start)
                .count()
        };
        
        // Calculate rate (events per second)
        let rate = events as f64 / window_seconds as f64;
        
        Ok(rate)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Event, EventType};
    
    #[tokio::test]
    async fn test_record_event() {
        let collector = MetricsCollector::new();
        
        let event = Event::new(
            EventType::SecurityAlert,
            "test",
            Some(serde_json::json!({"alert": "Test alert"})),
        );
        
        collector.record_event(&event).await.unwrap();
        
        assert_eq!(collector.get_counter("event_count_SecurityAlert"), 1);
        assert_eq!(collector.get_counter("event_source_test"), 1);
    }
    
    #[tokio::test]
    async fn test_counters() {
        let collector = MetricsCollector::new();
        
        collector.set_counter("test_counter", 42);
        assert_eq!(collector.get_counter("test_counter"), 42);
        
        collector.increment_counter("test_counter");
        assert_eq!(collector.get_counter("test_counter"), 43);
    }
}
