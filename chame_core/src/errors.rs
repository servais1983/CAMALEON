use thiserror::Error;

/// Error types for CAMALEON
#[derive(Error, Debug)]
pub enum ChameleonError {
    #[error("Initialization error: {0}")]
    InitializationError(String),
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Posture change error: {0}")]
    PostureChangeError(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ChameleonError {
    pub fn new_init_error(msg: impl Into<String>) -> Self {
        Self::InitializationError(msg.into())
    }
    
    pub fn new_runtime_error(msg: impl Into<String>) -> Self {
        Self::RuntimeError(msg.into())
    }
    
    pub fn new_config_error(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }
    
    pub fn new_network_error(msg: impl Into<String>) -> Self {
        Self::NetworkError(msg.into())
    }
    
    pub fn new_system_error(msg: impl Into<String>) -> Self {
        Self::SystemError(msg.into())
    }
    
    pub fn new_posture_error(msg: impl Into<String>) -> Self {
        Self::PostureChangeError(msg.into())
    }
    
    pub fn new_unavailable_error(msg: impl Into<String>) -> Self {
        Self::ServiceUnavailable(msg.into())
    }
    
    pub fn new_invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState(msg.into())
    }
    
    pub fn new_invalid_operation(msg: impl Into<String>) -> Self {
        Self::InvalidOperation(msg.into())
    }
    
    pub fn new_unknown_error(msg: impl Into<String>) -> Self {
        Self::Unknown(msg.into())
    }
}
