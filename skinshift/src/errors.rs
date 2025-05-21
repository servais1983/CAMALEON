use chame_core::ChameleonError;
use thiserror::Error;

/// Error types specific to the skinshift module
#[derive(Error, Debug)]
pub enum SkinshiftError {
    #[error("Fingerprint error: {0}")]
    FingerprintError(String),
    
    #[error("Banner error: {0}")]
    BannerError(String),
    
    #[error("Firewall error: {0}")]
    FirewallError(String),
    
    #[error("Preset error: {0}")]
    PresetError(String),
    
    #[error("Service error: {0}")]
    ServiceError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Process execution error: {0}")]
    ProcessError(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Implement conversion from SkinshiftError to ChameleonError
impl From<SkinshiftError> for ChameleonError {
    fn from(error: SkinshiftError) -> Self {
        match error {
            SkinshiftError::FingerprintError(msg) => ChameleonError::SystemError(format!("Fingerprint: {}", msg)),
            SkinshiftError::BannerError(msg) => ChameleonError::SystemError(format!("Banner: {}", msg)),
            SkinshiftError::FirewallError(msg) => ChameleonError::SystemError(format!("Firewall: {}", msg)),
            SkinshiftError::PresetError(msg) => ChameleonError::ConfigError(format!("Preset: {}", msg)),
            SkinshiftError::ServiceError(msg) => ChameleonError::ServiceUnavailable(msg),
            SkinshiftError::ConfigError(msg) => ChameleonError::ConfigError(msg),
            SkinshiftError::IOError(e) => ChameleonError::IOError(e),
            SkinshiftError::SerializationError(e) => ChameleonError::SerializationError(e.to_string()),
            SkinshiftError::PermissionDenied(msg) => ChameleonError::SystemError(format!("Permission denied: {}", msg)),
            SkinshiftError::ProcessError(msg) => ChameleonError::SystemError(format!("Process error: {}", msg)),
            SkinshiftError::SystemError(msg) => ChameleonError::SystemError(msg),
            SkinshiftError::NotImplemented(msg) => ChameleonError::InvalidOperation(format!("Not implemented: {}", msg)),
        }
    }
}
