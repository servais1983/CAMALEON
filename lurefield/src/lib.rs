use chame_core::events::{Event, EventType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Errors that can occur in the Lurefield module
#[derive(Error, Debug)]
pub enum LurefieldError {
    #[error("Honeypot creation error: {0}")]
    HoneypotCreation(String),
    
    #[error("Honeypot deployment error: {0}")]
    HoneypotDeployment(String),
    
    #[error("Template error: {0}")]
    Template(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Maximum honeypots reached")]
    MaxHoneypotsReached,
}

/// Configuration for the Lurefield module
#[derive(Debug, Clone)]
pub struct LurefieldConfig {
    /// Directory containing honeypot templates
    pub honeypot_dir: PathBuf,
    
    /// Maximum number of honeypots to deploy simultaneously
    pub max_honeypots: u32,
    
    /// Whether to automatically deploy honeypots
    pub auto_deploy: bool,
}

impl Default for LurefieldConfig {
    fn default() -> Self {
        Self {
            honeypot_dir: PathBuf::from("./honeypots"),
            max_honeypots: 5,
            auto_deploy: false,
        }
    }
}

/// Types of honeypots that can be deployed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoneypotType {
    /// SSH server honeypot
    Ssh,
    
    /// HTTP server honeypot
    Http,
    
    /// FTP server honeypot
    Ftp,
    
    /// SMB server honeypot
    Smb,
    
    /// Database server honeypot
    Database(String), // e.g., "mysql", "mongodb"
    
    /// Custom honeypot
    Custom(String),
}

impl HoneypotType {
    /// Convert a string to a honeypot type
    pub fn from_str(s: &str) -> Result<Self, LurefieldError> {
        match s.to_lowercase().as_str() {
            "ssh" => Ok(Self::Ssh),
            "http" => Ok(Self::Http),
            "ftp" => Ok(Self::Ftp),
            "smb" => Ok(Self::Smb),
            s if s.starts_with("db:") || s.starts_with("database:") => {
                let db_type = s.split(':').nth(1).unwrap_or("generic");
                Ok(Self::Database(db_type.to_string()))
            }
            _ => Ok(Self::Custom(s.to_string())),
        }
    }
    
    /// Get the default port for this honeypot type
    pub fn default_port(&self) -> u16 {
        match self {
            Self::Ssh => 22,
            Self::Http => 80,
            Self::Ftp => 21,
            Self::Smb => 445,
            Self::Database(db_type) => match db_type.as_str() {
                "mysql" => 3306,
                "mongodb" => 27017,
                "postgresql" => 5432,
                "redis" => 6379,
                _ => 27017, // Default to MongoDB port
            },
            Self::Custom(_) => 8080,
        }
    }
}

/// Options for honeypot deployment
#[derive(Debug, Clone)]
pub struct HoneypotOptions {
    /// Port to listen on
    pub port: u16,
    
    /// Whether to enable fake authentication
    pub fake_auth: bool,
    
    /// Whether to log keystrokes
    pub log_keystroke: bool,
    
    /// Custom banner or response
    pub custom_banner: Option<String>,
    
    /// Additional options
    pub extra_options: HashMap<String, String>,
}

impl Default for HoneypotOptions {
    fn default() -> Self {
        Self {
            port: 0, // Will be set based on honeypot type
            fake_auth: true,
            log_keystroke: true,
            custom_banner: None,
            extra_options: HashMap::new(),
        }
    }
}

/// A deployed honeypot
#[derive(Debug)]
pub struct Honeypot {
    /// Unique ID
    pub id: String,
    
    /// Type of honeypot
    pub honeypot_type: HoneypotType,
    
    /// Port the honeypot is listening on
    pub port: u16,
    
    /// Options used for deployment
    pub options: HoneypotOptions,
    
    /// When the honeypot was deployed
    pub deployed_at: chrono::DateTime<chrono::Utc>,
    
    /// Number of interactions with the honeypot
    pub interaction_count: u32,
    
    /// Whether the honeypot is currently active
    pub active: bool,
    
    /// Process handle (if applicable)
    #[allow(dead_code)]
    process_handle: Option<tokio::process::Child>,
}

/// Main Lurefield honeypot management service
pub struct Lurefield {
    /// Configuration
    config: LurefieldConfig,
    
    /// Active honeypots
    honeypots: RwLock<HashMap<String, Arc<RwLock<Honeypot>>>>,
    
    /// Event sender
    event_sender: tokio::sync::mpsc::Sender<Event>,
    
    /// Template engine
    template_engine: handlebars::Handlebars<'static>,
}

impl Lurefield {
    /// Create a new Lurefield instance
    pub async fn new(
        config: LurefieldConfig,
        event_sender: tokio::sync::mpsc::Sender<Event>,
    ) -> Result<Self, LurefieldError> {
        // Create honeypot directory if it doesn't exist
        if !config.honeypot_dir.exists() {
            tokio::fs::create_dir_all(&config.honeypot_dir).await?;
        }
        
        // Initialize template engine
        let mut template_engine = handlebars::Handlebars::new();
        template_engine.set_strict_mode(true);
        
        // Load templates
        if let Ok(entries) = std::fs::read_dir(&config.honeypot_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "hbs") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(template) = std::fs::read_to_string(&path) {
                            if let Err(e) = template_engine.register_template_string(name, template) {
                                tracing::warn!("Failed to register template {}: {}", name, e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(Self {
            config,
            honeypots: RwLock::new(HashMap::new()),
            event_sender,
            template_engine,
        })
    }
    
    /// Start the Lurefield service
    pub async fn start(&self) -> Result<(), LurefieldError> {
        tracing::info!("Starting Lurefield honeypot service");
        
        // Auto-deploy honeypots if configured
        if self.config.auto_deploy {
            self.auto_deploy_honeypots().await?;
        }
        
        Ok(())
    }
    
    /// Stop the Lurefield service
    pub async fn stop(&self) -> Result<(), LurefieldError> {
        tracing::info!("Stopping Lurefield honeypot service");
        
        // Stop all honeypots
        let honeypots = self.honeypots.read().await;
        for (id, honeypot) in honeypots.iter() {
            if let Err(e) = self.stop_honeypot(id).await {
                tracing::warn!("Failed to stop honeypot {}: {}", id, e);
            }
        }
        
        Ok(())
    }
    
    /// Deploy a new honeypot
    pub async fn deploy_honeypot(
        &self,
        honeypot_type: HoneypotType,
        options: Option<HoneypotOptions>,
    ) -> Result<String, LurefieldError> {
        // Check if we've reached the maximum number of honeypots
        let honeypots = self.honeypots.read().await;
        if honeypots.len() >= self.config.max_honeypots as usize {
            return Err(LurefieldError::MaxHoneypotsReached);
        }
        drop(honeypots);
        
        // Generate a unique ID
        let id = format!(
            "hp-{}-{}",
            honeypot_type.to_str(),
            chrono::Utc::now().timestamp()
        );
        
        // Prepare options
        let mut options = options.unwrap_or_default();
        if options.port == 0 {
            options.port = honeypot_type.default_port();
        }
        
        // Create the honeypot
        let honeypot = Honeypot {
            id: id.clone(),
            honeypot_type: honeypot_type.clone(),
            port: options.port,
            options: options.clone(),
            deployed_at: chrono::Utc::now(),
            interaction_count: 0,
            active: true,
            process_handle: None,
        };
        
        // Store the honeypot
        {
            let mut honeypots = self.honeypots.write().await;
            honeypots.insert(id.clone(), Arc::new(RwLock::new(honeypot)));
        }
        
        // Send event
        let event = Event::honeypot_activity(
            "lurefield",
            Some(serde_json::json!({
                "action": "deploy",
                "honeypot_id": id,
                "honeypot_type": honeypot_type.to_str(),
                "port": options.port,
            })),
        );
        
        if let Err(e) = self.event_sender.send(event).await {
            tracing::error!("Failed to send honeypot deployment event: {}", e);
        }
        
        tracing::info!(
            "Deployed {} honeypot on port {}",
            honeypot_type.to_str(),
            options.port
        );
        
        Ok(id)
    }
    
    /// Stop a honeypot
    pub async fn stop_honeypot(&self, id: &str) -> Result<(), LurefieldError> {
        let honeypots = self.honeypots.read().await;
        let honeypot_lock = honeypots.get(id).ok_or_else(|| {
            LurefieldError::HoneypotDeployment(format!("Honeypot {} not found", id))
        })?;
        
        // Mark as inactive
        {
            let mut honeypot = honeypot_lock.write().await;
            honeypot.active = false;
        }
        
        // Send event
        let event = Event::honeypot_activity(
            "lurefield",
            Some(serde_json::json!({
                "action": "stop",
                "honeypot_id": id,
            })),
        );
        
        if let Err(e) = self.event_sender.send(event).await {
            tracing::error!("Failed to send honeypot stop event: {}", e);
        }
        
        tracing::info!("Stopped honeypot {}", id);
        
        Ok(())
    }
    
    /// Get all active honeypots
    pub async fn get_honeypots(&self) -> HashMap<String, Honeypot> {
        let honeypots = self.honeypots.read().await;
        let mut result = HashMap::new();
        
        for (id, honeypot_lock) in honeypots.iter() {
            let honeypot = honeypot_lock.read().await;
            if honeypot.active {
                result.insert(id.clone(), Honeypot {
                    id: honeypot.id.clone(),
                    honeypot_type: honeypot.honeypot_type.clone(),
                    port: honeypot.port,
                    options: honeypot.options.clone(),
                    deployed_at: honeypot.deployed_at,
                    interaction_count: honeypot.interaction_count,
                    active: honeypot.active,
                    process_handle: None,
                });
            }
        }
        
        result
    }
    
    /// Record an interaction with a honeypot
    pub async fn record_interaction(
        &self,
        id: &str,
        details: HashMap<String, String>,
    ) -> Result<(), LurefieldError> {
        let honeypots = self.honeypots.read().await;
        let honeypot_lock = honeypots.get(id).ok_or_else(|| {
            LurefieldError::HoneypotDeployment(format!("Honeypot {} not found", id))
        })?;
        
        // Increment interaction count
        {
            let mut honeypot = honeypot_lock.write().await;
            honeypot.interaction_count += 1;
        }
        
        // Send event
        let event = Event::honeypot_activity(
            "lurefield",
            Some(serde_json::json!({
                "action": "interaction",
                "honeypot_id": id,
                "details": details,
            })),
        );
        
        if let Err(e) = self.event_sender.send(event).await {
            tracing::error!("Failed to send honeypot interaction event: {}", e);
        }
        
        Ok(())
    }
    
    /// Auto-deploy honeypots based on configuration
    async fn auto_deploy_honeypots(&self) -> Result<(), LurefieldError> {
        // Deploy a basic set of honeypots
        self.deploy_honeypot(HoneypotType::Ssh, None).await?;
        self.deploy_honeypot(HoneypotType::Http, None).await?;
        self.deploy_honeypot(HoneypotType::Database("mongodb".to_string()), None).await?;
        
        Ok(())
    }
}

impl HoneypotType {
    /// Convert a honeypot type to a string
    pub fn to_str(&self) -> &str {
        match self {
            Self::Ssh => "ssh",
            Self::Http => "http",
            Self::Ftp => "ftp",
            Self::Smb => "smb",
            Self::Database(db_type) => db_type,
            Self::Custom(name) => name,
        }
    }
}
