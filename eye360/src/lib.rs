use chame_core::events::{Event, EventType};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Errors that can occur in the Eye360 module
#[derive(Error, Debug)]
pub enum Eye360Error {
    #[error("System monitoring error: {0}")]
    SystemMonitoring(String),
    
    #[error("Syscall tracking error: {0}")]
    SyscallTracking(String),
    
    #[error("eBPF error: {0}")]
    Ebpf(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration for the Eye360 module
#[derive(Debug, Clone)]
pub struct Eye360Config {
    /// Whether system call monitoring is enabled
    pub syscall_monitoring: bool,
    
    /// Whether to log suspicious activities
    pub log_suspicious: bool,
    
    /// Whether eBPF monitoring is enabled
    pub ebpf_enabled: bool,
    
    /// List of syscalls to monitor specifically
    pub monitored_syscalls: Vec<String>,
}

impl Default for Eye360Config {
    fn default() -> Self {
        Self {
            syscall_monitoring: true,
            log_suspicious: true,
            ebpf_enabled: false,
            monitored_syscalls: vec![
                "execve".to_string(),
                "fork".to_string(),
                "clone".to_string(),
                "open".to_string(),
                "connect".to_string(),
                "bind".to_string(),
                "socket".to_string(),
            ],
        }
    }
}

/// A detection for suspicious system activity
#[derive(Debug, Clone)]
pub struct Detection {
    /// Type of detection
    pub detection_type: DetectionType,
    
    /// Source of the detection
    pub source: String,
    
    /// Details about the detection
    pub details: HashMap<String, String>,
    
    /// Severity level (0-10)
    pub severity: u8,
    
    /// Timestamp of the detection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of system detections
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionType {
    /// Suspicious syscall activity
    SuspiciousSyscall,
    
    /// Unusual process behavior
    UnusualProcess,
    
    /// File system anomaly
    FileSystemAnomaly,
    
    /// Network connection anomaly
    NetworkAnomaly,
    
    /// Privilege escalation attempt
    PrivilegeEscalation,
    
    /// Other detection types
    Other(String),
}

/// Main Eye360 system monitoring service
pub struct Eye360 {
    /// Configuration
    config: Eye360Config,
    
    /// Detection history
    detections: RwLock<Vec<Detection>>,
    
    /// Event sender
    event_sender: tokio::sync::mpsc::Sender<Event>,
    
    /// Process monitor
    process_monitor: Option<Arc<ProcessMonitor>>,
    
    /// Syscall monitor
    syscall_monitor: Option<Arc<SyscallMonitor>>,
    
    /// eBPF monitor
    ebpf_monitor: Option<Arc<EbpfMonitor>>,
}

impl Eye360 {
    /// Create a new Eye360 instance
    pub async fn new(
        config: Eye360Config,
        event_sender: tokio::sync::mpsc::Sender<Event>,
    ) -> Result<Self, Eye360Error> {
        let process_monitor = Some(Arc::new(ProcessMonitor::new()?));
        
        let syscall_monitor = if config.syscall_monitoring {
            Some(Arc::new(SyscallMonitor::new(&config.monitored_syscalls)?))
        } else {
            None
        };
        
        let ebpf_monitor = if config.ebpf_enabled {
            match EbpfMonitor::new() {
                Ok(monitor) => Some(Arc::new(monitor)),
                Err(e) => {
                    tracing::warn!("Failed to initialize eBPF monitor: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Ok(Self {
            config,
            detections: RwLock::new(Vec::new()),
            event_sender,
            process_monitor,
            syscall_monitor,
            ebpf_monitor,
        })
    }
    
    /// Start monitoring
    pub async fn start(&self) -> Result<(), Eye360Error> {
        tracing::info!("Starting Eye360 system monitoring");
        
        // Start process monitoring
        if let Some(monitor) = &self.process_monitor {
            monitor.start().await?;
        }
        
        // Start syscall monitoring if enabled
        if let Some(monitor) = &self.syscall_monitor {
            monitor.start().await?;
        }
        
        // Start eBPF monitoring if enabled
        if let Some(monitor) = &self.ebpf_monitor {
            monitor.start().await?;
        }
        
        Ok(())
    }
    
    /// Stop monitoring
    pub async fn stop(&self) -> Result<(), Eye360Error> {
        tracing::info!("Stopping Eye360 system monitoring");
        
        // Stop process monitoring
        if let Some(monitor) = &self.process_monitor {
            monitor.stop().await?;
        }
        
        // Stop syscall monitoring if enabled
        if let Some(monitor) = &self.syscall_monitor {
            monitor.stop().await?;
        }
        
        // Stop eBPF monitoring if enabled
        if let Some(monitor) = &self.ebpf_monitor {
            monitor.stop().await?;
        }
        
        Ok(())
    }
    
    /// Add a detection
    pub async fn add_detection(&self, detection: Detection) -> Result<(), Eye360Error> {
        // Add to history
        {
            let mut detections = self.detections.write().await;
            detections.push(detection.clone());
        }
        
        // Send event
        let event = Event::security_alert(
            "eye360",
            Some(serde_json::to_value(&detection).unwrap_or_default()),
        );
        
        if let Err(e) = self.event_sender.send(event).await {
            tracing::error!("Failed to send detection event: {}", e);
        }
        
        Ok(())
    }
    
    /// Get detection history
    pub async fn get_detections(&self) -> Vec<Detection> {
        let detections = self.detections.read().await;
        detections.clone()
    }
}

/// Monitor for system processes
pub struct ProcessMonitor {
    /// Whether the monitor is running
    running: RwLock<bool>,
}

impl ProcessMonitor {
    /// Create a new process monitor
    pub fn new() -> Result<Self, Eye360Error> {
        Ok(Self {
            running: RwLock::new(false),
        })
    }
    
    /// Start monitoring
    pub async fn start(&self) -> Result<(), Eye360Error> {
        let mut running = self.running.write().await;
        *running = true;
        tracing::info!("Process monitoring started");
        Ok(())
    }
    
    /// Stop monitoring
    pub async fn stop(&self) -> Result<(), Eye360Error> {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Process monitoring stopped");
        Ok(())
    }
}

/// Monitor for system calls
pub struct SyscallMonitor {
    /// Whether the monitor is running
    running: RwLock<bool>,
    
    /// Syscalls to monitor
    syscalls: Vec<String>,
}

impl SyscallMonitor {
    /// Create a new syscall monitor
    pub fn new(syscalls: &[String]) -> Result<Self, Eye360Error> {
        Ok(Self {
            running: RwLock::new(false),
            syscalls: syscalls.to_vec(),
        })
    }
    
    /// Start monitoring
    pub async fn start(&self) -> Result<(), Eye360Error> {
        let mut running = self.running.write().await;
        *running = true;
        tracing::info!("Syscall monitoring started for: {:?}", self.syscalls);
        Ok(())
    }
    
    /// Stop monitoring
    pub async fn stop(&self) -> Result<(), Eye360Error> {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Syscall monitoring stopped");
        Ok(())
    }
}

/// Monitor using eBPF
pub struct EbpfMonitor {
    /// Whether the monitor is running
    running: RwLock<bool>,
}

impl EbpfMonitor {
    /// Create a new eBPF monitor
    pub fn new() -> Result<Self, Eye360Error> {
        // Check if we have root permissions
        if !nix::unistd::geteuid().is_root() {
            return Err(Eye360Error::PermissionDenied(
                "eBPF monitoring requires root permissions".to_string(),
            ));
        }
        
        Ok(Self {
            running: RwLock::new(false),
        })
    }
    
    /// Start monitoring
    pub async fn start(&self) -> Result<(), Eye360Error> {
        let mut running = self.running.write().await;
        *running = true;
        tracing::info!("eBPF monitoring started");
        Ok(())
    }
    
    /// Stop monitoring
    pub async fn stop(&self) -> Result<(), Eye360Error> {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("eBPF monitoring stopped");
        Ok(())
    }
}
