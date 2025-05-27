use chame_core::events::{Event, EventType};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Errors that can occur in the NetTongue module
#[derive(Error, Debug)]
pub enum NetTongueError {
    #[error("Network monitoring error: {0}")]
    NetworkMonitoring(String),
    
    #[error("Packet capture error: {0}")]
    PacketCapture(String),
    
    #[error("Latency fuzzing error: {0}")]
    LatencyFuzz(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("PCAP error: {0}")]
    Pcap(String),
}

/// Configuration for the NetTongue module
#[derive(Debug, Clone)]
pub struct NetTongueConfig {
    /// Whether packet capture is enabled
    pub pcap_enabled: bool,
    
    /// Network interface to monitor
    pub interface: String,
    
    /// Whether latency fuzzing is enabled
    pub latency_fuzz_enabled: bool,
    
    /// Minimum latency fuzz in milliseconds
    pub latency_fuzz_min_ms: u64,
    
    /// Maximum latency fuzz in milliseconds
    pub latency_fuzz_max_ms: u64,
}

impl Default for NetTongueConfig {
    fn default() -> Self {
        Self {
            pcap_enabled: true,
            interface: "eth0".to_string(),
            latency_fuzz_enabled: false,
            latency_fuzz_min_ms: 50,
            latency_fuzz_max_ms: 200,
        }
    }
}

/// A network detection
#[derive(Debug, Clone)]
pub struct NetworkDetection {
    /// Type of detection
    pub detection_type: NetworkDetectionType,
    
    /// Source IP address
    pub source_ip: Option<String>,
    
    /// Destination IP address
    pub dest_ip: Option<String>,
    
    /// Source port
    pub source_port: Option<u16>,
    
    /// Destination port
    pub dest_port: Option<u16>,
    
    /// Protocol
    pub protocol: Option<String>,
    
    /// Details about the detection
    pub details: HashMap<String, String>,
    
    /// Severity level (0-10)
    pub severity: u8,
    
    /// Timestamp of the detection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of network detections
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkDetectionType {
    /// Port scan detection
    PortScan,
    
    /// SYN flood detection
    SynFlood,
    
    /// Unusual connection pattern
    UnusualConnectionPattern,
    
    /// Protocol anomaly
    ProtocolAnomaly,
    
    /// Fingerprinting attempt
    FingerprintingAttempt,
    
    /// Other detection types
    Other(String),
}

/// Main NetTongue network monitoring service
pub struct NetTongue {
    /// Configuration
    config: NetTongueConfig,
    
    /// Detection history
    detections: RwLock<Vec<NetworkDetection>>,
    
    /// Event sender
    event_sender: tokio::sync::mpsc::Sender<Event>,
    
    /// Packet capture monitor
    pcap_monitor: Option<Arc<PcapMonitor>>,
    
    /// Latency fuzzer
    latency_fuzzer: Option<Arc<LatencyFuzzer>>,
}

impl NetTongue {
    /// Create a new NetTongue instance
    pub async fn new(
        config: NetTongueConfig,
        event_sender: tokio::sync::mpsc::Sender<Event>,
    ) -> Result<Self, NetTongueError> {
        let pcap_monitor = if config.pcap_enabled {
            match PcapMonitor::new(&config.interface) {
                Ok(monitor) => Some(Arc::new(monitor)),
                Err(e) => {
                    tracing::warn!("Failed to initialize PCAP monitor: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        let latency_fuzzer = if config.latency_fuzz_enabled {
            Some(Arc::new(LatencyFuzzer::new(
                config.latency_fuzz_min_ms,
                config.latency_fuzz_max_ms,
            )))
        } else {
            None
        };
        
        Ok(Self {
            config,
            detections: RwLock::new(Vec::new()),
            event_sender,
            pcap_monitor,
            latency_fuzzer,
        })
    }
    
    /// Start monitoring
    pub async fn start(&self) -> Result<(), NetTongueError> {
        tracing::info!("Starting NetTongue network monitoring");
        
        // Start packet capture if enabled
        if let Some(monitor) = &self.pcap_monitor {
            monitor.start().await?;
        }
        
        // Start latency fuzzing if enabled
        if let Some(fuzzer) = &self.latency_fuzzer {
            fuzzer.start().await?;
        }
        
        Ok(())
    }
    
    /// Stop monitoring
    pub async fn stop(&self) -> Result<(), NetTongueError> {
        tracing::info!("Stopping NetTongue network monitoring");
        
        // Stop packet capture if enabled
        if let Some(monitor) = &self.pcap_monitor {
            monitor.stop().await?;
        }
        
        // Stop latency fuzzing if enabled
        if let Some(fuzzer) = &self.latency_fuzzer {
            fuzzer.stop().await?;
        }
        
        Ok(())
    }
    
    /// Add a detection
    pub async fn add_detection(&self, detection: NetworkDetection) -> Result<(), NetTongueError> {
        // Add to history
        {
            let mut detections = self.detections.write().await;
            detections.push(detection.clone());
        }
        
        // Send event
        let event = Event::network_activity(
            "nettongue",
            Some(serde_json::to_value(&detection).unwrap_or_default()),
        );
        
        if let Err(e) = self.event_sender.send(event).await {
            tracing::error!("Failed to send detection event: {}", e);
        }
        
        Ok(())
    }
    
    /// Get detection history
    pub async fn get_detections(&self) -> Vec<NetworkDetection> {
        let detections = self.detections.read().await;
        detections.clone()
    }
}

/// Monitor for packet capture
pub struct PcapMonitor {
    /// Network interface
    interface: String,
    
    /// Whether the monitor is running
    running: RwLock<bool>,
}

impl PcapMonitor {
    /// Create a new packet capture monitor
    pub fn new(interface: &str) -> Result<Self, NetTongueError> {
        // Check if the interface exists
        match pcap::Device::list() {
            Ok(devices) => {
                if !devices.iter().any(|d| d.name == interface) {
                    return Err(NetTongueError::PacketCapture(format!(
                        "Interface {} not found",
                        interface
                    )));
                }
            }
            Err(e) => {
                return Err(NetTongueError::Pcap(format!("Failed to list devices: {}", e)));
            }
        }
        
        Ok(Self {
            interface: interface.to_string(),
            running: RwLock::new(false),
        })
    }
    
    /// Start monitoring
    pub async fn start(&self) -> Result<(), NetTongueError> {
        let mut running = self.running.write().await;
        *running = true;
        tracing::info!("Packet capture started on interface {}", self.interface);
        Ok(())
    }
    
    /// Stop monitoring
    pub async fn stop(&self) -> Result<(), NetTongueError> {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Packet capture stopped");
        Ok(())
    }
}

/// Latency fuzzer for confusing timing attacks
pub struct LatencyFuzzer {
    /// Minimum latency in milliseconds
    min_ms: u64,
    
    /// Maximum latency in milliseconds
    max_ms: u64,
    
    /// Whether the fuzzer is running
    running: RwLock<bool>,
}

impl LatencyFuzzer {
    /// Create a new latency fuzzer
    pub fn new(min_ms: u64, max_ms: u64) -> Self {
        Self {
            min_ms,
            max_ms,
            running: RwLock::new(false),
        }
    }
    
    /// Start fuzzing
    pub async fn start(&self) -> Result<(), NetTongueError> {
        let mut running = self.running.write().await;
        *running = true;
        tracing::info!(
            "Latency fuzzing started (range: {}-{} ms)",
            self.min_ms,
            self.max_ms
        );
        Ok(())
    }
    
    /// Stop fuzzing
    pub async fn stop(&self) -> Result<(), NetTongueError> {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Latency fuzzing stopped");
        Ok(())
    }
    
    /// Get a random latency value
    pub fn get_latency(&self) -> u64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(self.min_ms..=self.max_ms)
    }
}
