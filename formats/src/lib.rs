use chame_core::events::{Event, EventType};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur in the Formats module
#[derive(Error, Debug)]
pub enum FormatsError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
}

/// Types of file formats supported
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFormat {
    /// CSV format
    Csv,
    
    /// Log format
    Log,
    
    /// VMDK format
    Vmdk,
    
    /// Unknown format
    Unknown,
}

impl FileFormat {
    /// Detect format from file extension
    pub fn from_extension(extension: &str) -> Self {
        match extension.to_lowercase().as_str() {
            "csv" => Self::Csv,
            "log" | "txt" => Self::Log,
            "vmdk" => Self::Vmdk,
            _ => Self::Unknown,
        }
    }
    
    /// Get format from file path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(Self::from_extension)
            .unwrap_or(Self::Unknown)
    }
}

/// Detection result from file analysis
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// Type of detection
    pub detection_type: String,
    
    /// Severity level (0-10)
    pub severity: u8,
    
    /// Location in file (line number, offset, etc.)
    pub location: String,
    
    /// Details about the detection
    pub details: HashMap<String, String>,
    
    /// Timestamp of the detection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// File analyzer trait
pub trait FileAnalyzer {
    /// Analyze a file and return detections
    fn analyze<P: AsRef<Path>>(&self, path: P) -> Result<Vec<DetectionResult>, FormatsError>;
    
    /// Get supported file format
    fn supported_format(&self) -> FileFormat;
}

/// Main Formats service
pub struct Formats {
    /// Registered analyzers
    analyzers: Vec<Box<dyn FileAnalyzer + Send + Sync>>,
    
    /// Event sender
    event_sender: tokio::sync::mpsc::Sender<Event>,
}

impl Formats {
    /// Create a new Formats instance
    pub fn new(event_sender: tokio::sync::mpsc::Sender<Event>) -> Self {
        let mut formats = Self {
            analyzers: Vec::new(),
            event_sender,
        };
        
        // Register default analyzers
        formats.register_analyzer(Box::new(CsvAnalyzer::new()));
        formats.register_analyzer(Box::new(LogAnalyzer::new()));
        
        formats
    }
    
    /// Register a file analyzer
    pub fn register_analyzer(&mut self, analyzer: Box<dyn FileAnalyzer + Send + Sync>) {
        self.analyzers.push(analyzer);
    }
    
    /// Analyze a file
    pub async fn analyze_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<DetectionResult>, FormatsError> {
        let path_ref = path.as_ref();
        
        // Check if file exists
        if !path_ref.exists() {
            return Err(FormatsError::FileNotFound(
                path_ref.to_string_lossy().to_string(),
            ));
        }
        
        // Detect format
        let format = FileFormat::from_path(path_ref);
        
        // Find analyzer
        let analyzer = self
            .analyzers
            .iter()
            .find(|a| a.supported_format() == format)
            .ok_or_else(|| {
                FormatsError::InvalidFormat(format!("No analyzer for format: {:?}", format))
            })?;
        
        // Analyze file
        let results = analyzer.analyze(path_ref)?;
        
        // Send events for detections
        for result in &results {
            let event = Event::security_alert(
                "formats",
                Some(serde_json::json!({
                    "detection_type": result.detection_type,
                    "severity": result.severity,
                    "location": result.location,
                    "details": result.details,
                    "file": path_ref.to_string_lossy(),
                })),
            );
            
            if let Err(e) = self.event_sender.send(event).await {
                tracing::error!("Failed to send detection event: {}", e);
            }
        }
        
        Ok(results)
    }
}

/// CSV file analyzer
pub struct CsvAnalyzer {
    /// Patterns to look for
    patterns: Vec<(regex::Regex, String, u8)>,
}

impl CsvAnalyzer {
    /// Create a new CSV analyzer
    pub fn new() -> Self {
        let mut analyzer = Self {
            patterns: Vec::new(),
        };
        
        // Add default patterns
        analyzer.add_pattern(r"(?i)bitlocker", "encryption_bitlocker", 8);
        analyzer.add_pattern(r"(?i)ransom", "ransomware_indicator", 9);
        analyzer.add_pattern(r"(?i)lockbit", "ransomware_lockbit", 10);
        analyzer.add_pattern(r"(?i)phish", "phishing_indicator", 7);
        analyzer.add_pattern(r"(?i)backdoor", "backdoor_indicator", 9);
        analyzer.add_pattern(r"(?i)malware", "malware_indicator", 8);
        analyzer.add_pattern(r"(?i)exploit", "exploit_indicator", 8);
        analyzer.add_pattern(r"(?i)suspicious", "suspicious_activity", 5);
        
        analyzer
    }
    
    /// Add a pattern to look for
    pub fn add_pattern(&mut self, pattern: &str, detection_type: &str, severity: u8) {
        if let Ok(regex) = regex::Regex::new(pattern) {
            self.patterns.push((regex, detection_type.to_string(), severity));
        }
    }
}

impl FileAnalyzer for CsvAnalyzer {
    fn analyze<P: AsRef<Path>>(&self, path: P) -> Result<Vec<DetectionResult>, FormatsError> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut results = Vec::new();
        
        for (row_idx, result) in reader.records().enumerate() {
            let record = result?;
            
            for (col_idx, field) in record.iter().enumerate() {
                for (pattern, detection_type, severity) in &self.patterns {
                    if pattern.is_match(field) {
                        let mut details = HashMap::new();
                        details.insert("matched_text".to_string(), field.to_string());
                        details.insert("column".to_string(), col_idx.to_string());
                        
                        results.push(DetectionResult {
                            detection_type: detection_type.clone(),
                            severity: *severity,
                            location: format!("row:{},col:{}", row_idx + 1, col_idx + 1),
                            details,
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    fn supported_format(&self) -> FileFormat {
        FileFormat::Csv
    }
}

/// Log file analyzer
pub struct LogAnalyzer {
    /// Patterns to look for
    patterns: Vec<(regex::Regex, String, u8)>,
}

impl LogAnalyzer {
    /// Create a new Log analyzer
    pub fn new() -> Self {
        let mut analyzer = Self {
            patterns: Vec::new(),
        };
        
        // Add default patterns
        analyzer.add_pattern(r"(?i)failed login|authentication failure", "auth_failure", 6);
        analyzer.add_pattern(r"(?i)brute force|bruteforce", "brute_force_attempt", 8);
        analyzer.add_pattern(r"(?i)permission denied", "permission_denied", 5);
        analyzer.add_pattern(r"(?i)suspicious activity", "suspicious_activity", 7);
        analyzer.add_pattern(r"(?i)malware|virus|trojan", "malware_indicator", 9);
        analyzer.add_pattern(r"(?i)exploit|vulnerability", "exploit_attempt", 8);
        analyzer.add_pattern(r"(?i)backdoor", "backdoor_indicator", 9);
        analyzer.add_pattern(r"(?i)command injection|sql injection", "injection_attempt", 9);
        
        analyzer
    }
    
    /// Add a pattern to look for
    pub fn add_pattern(&mut self, pattern: &str, detection_type: &str, severity: u8) {
        if let Ok(regex) = regex::Regex::new(pattern) {
            self.patterns.push((regex, detection_type.to_string(), severity));
        }
    }
}

impl FileAnalyzer for LogAnalyzer {
    fn analyze<P: AsRef<Path>>(&self, path: P) -> Result<Vec<DetectionResult>, FormatsError> {
        let content = std::fs::read_to_string(path)?;
        let mut results = Vec::new();
        
        for (line_idx, line) in content.lines().enumerate() {
            for (pattern, detection_type, severity) in &self.patterns {
                if let Some(captures) = pattern.captures(line) {
                    let matched_text = captures.get(0).map_or("", |m| m.as_str());
                    
                    let mut details = HashMap::new();
                    details.insert("matched_text".to_string(), matched_text.to_string());
                    details.insert("full_line".to_string(), line.to_string());
                    
                    results.push(DetectionResult {
                        detection_type: detection_type.clone(),
                        severity: *severity,
                        location: format!("line:{}", line_idx + 1),
                        details,
                        timestamp: chrono::Utc::now(),
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    fn supported_format(&self) -> FileFormat {
        FileFormat::Log
    }
}
