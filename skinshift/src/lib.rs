mod banner;
mod errors;
mod fingerprint;
mod firewall;
mod preset;
mod service;

use async_trait::async_trait;
use banner::BannerManager;
use chame_core::{ChameleonError, ChameleonService, Event, Posture, SystemState};
use errors::SkinshiftError;
use fingerprint::FingerprintManager;
use firewall::FirewallManager;
use preset::PresetManager;
use service::ServiceManager;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Main Skinshift service for OS fingerprint and banner morphing
pub struct SkinshiftService {
    /// Fingerprint management
    fingerprint_manager: Arc<FingerprintManager>,
    
    /// Banner manipulation
    banner_manager: Arc<BannerManager>,
    
    /// Firewall rules management
    firewall_manager: Arc<FirewallManager>,
    
    /// Fingerprint preset management
    preset_manager: Arc<PresetManager>,
    
    /// Service management
    service_manager: Arc<ServiceManager>,
    
    /// Current posture
    current_posture: Arc<RwLock<Posture>>,
    
    /// Configuration directory
    config_dir: String,
}

impl SkinshiftService {
    /// Create a new Skinshift service
    pub async fn new(config_dir: impl Into<String>) -> Result<Self, SkinshiftError> {
        let config_dir = config_dir.into();
        
        // Initialize components
        let fingerprint_manager = Arc::new(FingerprintManager::new());
        let banner_manager = Arc::new(BannerManager::new());
        let firewall_manager = Arc::new(FirewallManager::new().await?);
        let preset_manager = Arc::new(PresetManager::new(&config_dir));
        let service_manager = Arc::new(ServiceManager::new());
        
        Ok(Self {
            fingerprint_manager,
            banner_manager,
            firewall_manager,
            preset_manager,
            service_manager,
            current_posture: Arc::new(RwLock::new(Posture::Neutral)),
            config_dir,
        })
    }
    
    /// Load a fingerprint preset
    pub async fn load_preset(&self, preset_name: &str) -> Result<(), SkinshiftError> {
        info!("Loading fingerprint preset: {}", preset_name);
        
        // Load the preset
        let preset = self.preset_manager.load_preset(preset_name).await?;
        
        // Apply OS fingerprint settings
        self.fingerprint_manager.apply_fingerprint(&preset.fingerprint).await?;
        
        // Apply banner changes
        for (service_name, banner) in &preset.banners {
            self.banner_manager.set_banner(service_name, banner).await?;
        }
        
        // Apply firewall rules
        if let Some(rules) = &preset.firewall_rules {
            self.firewall_manager.apply_rules(rules).await?;
        }
        
        // Configure service behavior
        for (service_name, config) in &preset.services {
            self.service_manager.configure_service(service_name, config).await?;
        }
        
        // Register the change event
        info!("Successfully applied preset: {}", preset_name);
        
        Ok(())
    }
    
    /// Create a custom fingerprint
    pub async fn create_custom_fingerprint(&self, path: &Path) -> Result<(), SkinshiftError> {
        info!("Creating custom fingerprint from: {}", path.display());
        
        // Load the custom configuration
        let custom_config = self.preset_manager.load_custom(path).await?;
        
        // Apply OS fingerprint
        self.fingerprint_manager.apply_fingerprint(&custom_config.fingerprint).await?;
        
        // Apply other settings
        // (Similar to load_preset, but from custom config)
        
        info!("Successfully applied custom fingerprint");
        
        Ok(())
    }
    
    /// Reset to default system fingerprint
    pub async fn reset_fingerprint(&self) -> Result<(), SkinshiftError> {
        info!("Resetting system fingerprint to default");
        
        // Reset OS fingerprint
        self.fingerprint_manager.reset().await?;
        
        // Reset banners
        self.banner_manager.reset_all().await?;
        
        // Reset firewall rules
        self.firewall_manager.reset().await?;
        
        // Reset service configurations
        self.service_manager.reset_all().await?;
        
        info!("Successfully reset system fingerprint");
        
        Ok(())
    }
    
    /// List available presets
    pub async fn list_presets(&self) -> Result<Vec<String>, SkinshiftError> {
        self.preset_manager.list_presets().await
    }
}

#[async_trait]
impl ChameleonService for SkinshiftService {
    async fn init(&self) -> Result<(), ChameleonError> {
        info!("Initializing Skinshift service");
        
        // Initialize fingerprint manager
        if let Err(e) = self.fingerprint_manager.init().await {
            error!("Failed to initialize fingerprint manager: {}", e);
            return Err(ChameleonError::InitializationError(format!(
                "Fingerprint manager initialization failed: {}", e
            )));
        }
        
        // Initialize banner manager
        if let Err(e) = self.banner_manager.init().await {
            error!("Failed to initialize banner manager: {}", e);
            return Err(ChameleonError::InitializationError(format!(
                "Banner manager initialization failed: {}", e
            )));
        }
        
        // Load default presets
        match self.preset_manager.init().await {
            Ok(_) => info!("Preset manager initialized successfully"),
            Err(e) => {
                warn!("Preset manager initialization warning: {}", e);
                // Non-fatal error, continue
            }
        }
        
        Ok(())
    }
    
    async fn start(&self) -> Result<(), ChameleonError> {
        info!("Starting Skinshift service");
        
        // Apply initial neutral posture
        self.apply_posture_fingerprint(Posture::Neutral).await?;
        
        Ok(())
    }
    
    async fn stop(&self) -> Result<(), ChameleonError> {
        info!("Stopping Skinshift service");
        
        // Optionally reset to default fingerprint on shutdown
        if let Err(e) = self.reset_fingerprint().await {
            warn!("Failed to reset fingerprint during shutdown: {}", e);
            // Non-fatal error, continue
        }
        
        Ok(())
    }
    
    async fn handle_event(&self, event: Event) -> Result<(), ChameleonError> {
        match event.event_type {
            chame_core::EventType::PostureChange => {
                if let Some(data) = &event.data {
                    // Extract the new posture from the event data
                    if let Some(posture_str) = data.get("new_posture").and_then(|p| p.as_str()) {
                        if let Some(posture) = Posture::from_str(posture_str) {
                            // Update our local tracking of posture
                            {
                                let mut current = self.current_posture.write().await;
                                *current = posture;
                            }
                            
                            // Apply the appropriate fingerprint for this posture
                            self.apply_posture_fingerprint(posture).await?;
                        }
                    }
                }
            }
            // Handle other event types as needed
            _ => {}
        }
        
        Ok(())
    }
    
    async fn change_posture(&self, posture: Posture) -> Result<(), ChameleonError> {
        // Update our tracking of current posture
        {
            let mut current = self.current_posture.write().await;
            *current = posture;
        }
        
        // Apply the appropriate fingerprint for this posture
        self.apply_posture_fingerprint(posture).await?;
        
        Ok(())
    }
    
    async fn get_state(&self) -> Result<SystemState, ChameleonError> {
        // This would typically be implemented to return Skinshift-specific state
        // For now, we'll return a minimal state
        let posture = {
            let posture = self.current_posture.read().await;
            *posture
        };
        
        let mut state = SystemState {
            status: chame_core::state::Status::Running,
            current_posture: posture,
            started_at: Some(chrono::Utc::now()),
            last_posture_change: None,
            threat_level: 0.0,
            active_services_count: 0,
            active_honeypots_count: 0,
            current_fingerprint: None,
        };
        
        // Add more detailed state as needed
        
        Ok(state)
    }
}

impl SkinshiftService {
    /// Apply the appropriate fingerprint for a given posture
    async fn apply_posture_fingerprint(&self, posture: Posture) -> Result<(), ChameleonError> {
        let preset_name = match posture {
            Posture::Silent => "silent_minimal",
            Posture::Neutral => "linux_standard",
            Posture::Mimetic => "windows_server2019", // Could be dynamic based on observed attacker interests
            Posture::Fulgurant => "router_vulnerable",
            Posture::Unstable => "random_changing",
        };
        
        info!("Applying {} fingerprint for posture: {:?}", preset_name, posture);
        
        if let Err(e) = self.load_preset(preset_name).await {
            error!("Failed to load preset '{}': {}", preset_name, e);
            return Err(ChameleonError::PostureChangeError(format!(
                "Failed to apply fingerprint for posture {:?}: {}", posture, e
            )));
        }
        
        Ok(())
    }
}
