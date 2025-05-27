use chame_core::events::{Event, EventType};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

/// Errors that can occur in the PigmentAPI module
#[derive(Error, Debug)]
pub enum PigmentApiError {
    #[error("API server error: {0}")]
    ServerError(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration for the PigmentAPI module
#[derive(Debug, Clone)]
pub struct PigmentApiConfig {
    /// Address to bind to
    pub bind_address: SocketAddr,
    
    /// Whether to enable CORS
    pub enable_cors: bool,
}

impl Default for PigmentApiConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".parse().unwrap(),
            enable_cors: true,
        }
    }
}

/// API response for system status
#[derive(Debug, Serialize)]
pub struct SystemStatusResponse {
    /// Current status
    pub status: String,
    
    /// Current posture
    pub posture: String,
    
    /// Active modules
    pub active_modules: Vec<String>,
    
    /// System metrics
    pub metrics: HashMap<String, serde_json::Value>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// API response for events
#[derive(Debug, Serialize)]
pub struct EventsResponse {
    /// Events
    pub events: Vec<EventInfo>,
    
    /// Total count
    pub total: usize,
    
    /// Page
    pub page: usize,
    
    /// Page size
    pub page_size: usize,
}

/// Event information
#[derive(Debug, Serialize)]
pub struct EventInfo {
    /// Event ID
    pub id: String,
    
    /// Event type
    pub event_type: String,
    
    /// Source
    pub source: String,
    
    /// Severity
    pub severity: String,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Data
    pub data: Option<serde_json::Value>,
}

/// API request to change posture
#[derive(Debug, Deserialize)]
pub struct ChangePostureRequest {
    /// New posture
    pub posture: String,
}

/// API response for posture change
#[derive(Debug, Serialize)]
pub struct ChangePostureResponse {
    /// Success status
    pub success: bool,
    
    /// Previous posture
    pub previous_posture: String,
    
    /// New posture
    pub new_posture: String,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Main PigmentAPI service
pub struct PigmentApi {
    /// Configuration
    config: PigmentApiConfig,
    
    /// Event sender
    event_sender: mpsc::Sender<Event>,
    
    /// Event receiver
    event_receiver: Arc<RwLock<mpsc::Receiver<Event>>>,
    
    /// Event history
    events: Arc<RwLock<Vec<Event>>>,
    
    /// Current posture
    current_posture: Arc<RwLock<String>>,
    
    /// Active modules
    active_modules: Arc<RwLock<HashMap<String, bool>>>,
    
    /// System metrics
    metrics: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl PigmentApi {
    /// Create a new PigmentAPI instance
    pub async fn new(
        config: PigmentApiConfig,
        event_sender: mpsc::Sender<Event>,
        event_receiver: mpsc::Receiver<Event>,
    ) -> Result<Self, PigmentApiError> {
        Ok(Self {
            config,
            event_sender,
            event_receiver: Arc::new(RwLock::new(event_receiver)),
            events: Arc::new(RwLock::new(Vec::new())),
            current_posture: Arc::new(RwLock::new("neutral".to_string())),
            active_modules: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Start the API server
    pub async fn start(&self) -> Result<(), PigmentApiError> {
        tracing::info!("Starting PigmentAPI server on {}", self.config.bind_address);
        
        // Start event listener
        self.start_event_listener().await;
        
        // Create router
        let router = self.create_router().await;
        
        // Start server
        let server = axum::Server::bind(&self.config.bind_address)
            .serve(router.into_make_service());
        
        // Run the server
        if let Err(e) = server.await {
            return Err(PigmentApiError::ServerError(format!("Server error: {}", e)));
        }
        
        Ok(())
    }
    
    /// Create the API router
    async fn create_router(&self) -> Router {
        // Create state
        let state = AppState {
            events: self.events.clone(),
            current_posture: self.current_posture.clone(),
            active_modules: self.active_modules.clone(),
            metrics: self.metrics.clone(),
            event_sender: self.event_sender.clone(),
        };
        
        // Create CORS layer if enabled
        let cors = if self.config.enable_cors {
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        } else {
            CorsLayer::permissive()
        };
        
        // Create router
        Router::new()
            .route("/api/status", get(get_status))
            .route("/api/events", get(get_events))
            .route("/api/posture", get(get_posture))
            .route("/api/posture", post(change_posture))
            .route("/api/modules", get(get_modules))
            .route("/api/modules/:name", post(toggle_module))
            .route("/api/metrics", get(get_metrics))
            .layer(cors)
            .with_state(state)
    }
    
    /// Start the event listener
    async fn start_event_listener(&self) {
        let events = self.events.clone();
        let current_posture = self.current_posture.clone();
        let active_modules = self.active_modules.clone();
        let metrics = self.metrics.clone();
        let mut event_receiver = self.event_receiver.write().await;
        
        tokio::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                // Store event
                {
                    let mut events_lock = events.write().await;
                    events_lock.push(event.clone());
                    
                    // Limit history size
                    if events_lock.len() > 1000 {
                        events_lock.remove(0);
                    }
                }
                
                // Update posture if it's a posture change event
                if let EventType::PostureChange = event.event_type {
                    if let Some(data) = &event.data {
                        if let Some(posture) = data.get("posture").and_then(|p| p.as_str()) {
                            let mut posture_lock = current_posture.write().await;
                            *posture_lock = posture.to_string();
                        }
                    }
                }
                
                // Update module status if it's a service lifecycle event
                if let EventType::ServiceLifecycle = event.event_type {
                    if let Some(data) = &event.data {
                        if let (Some(module), Some(status)) = (
                            data.get("module").and_then(|m| m.as_str()),
                            data.get("status").and_then(|s| s.as_str()),
                        ) {
                            let mut modules_lock = active_modules.write().await;
                            modules_lock.insert(module.to_string(), status == "active");
                        }
                    }
                }
                
                // Update metrics if it's a metrics report event
                if let EventType::MetricsReport = event.event_type {
                    if let Some(data) = &event.data {
                        if let Some(metrics_data) = data.get("metrics") {
                            let mut metrics_lock = metrics.write().await;
                            if let Some(obj) = metrics_data.as_object() {
                                for (key, value) in obj {
                                    metrics_lock.insert(key.clone(), value.clone());
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

/// Application state
#[derive(Clone)]
struct AppState {
    /// Event history
    events: Arc<RwLock<Vec<Event>>>,
    
    /// Current posture
    current_posture: Arc<RwLock<String>>,
    
    /// Active modules
    active_modules: Arc<RwLock<HashMap<String, bool>>>,
    
    /// System metrics
    metrics: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    
    /// Event sender
    event_sender: mpsc::Sender<Event>,
}

/// Get system status
async fn get_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let posture = state.current_posture.read().await.clone();
    let active_modules = state.active_modules.read().await;
    let metrics = state.metrics.read().await;
    
    let active_module_names = active_modules
        .iter()
        .filter_map(|(name, active)| if *active { Some(name.clone()) } else { None })
        .collect();
    
    let response = SystemStatusResponse {
        status: "running".to_string(),
        posture,
        active_modules: active_module_names,
        metrics: metrics.clone(),
        timestamp: chrono::Utc::now(),
    };
    
    (StatusCode::OK, Json(response))
}

/// Query parameters for events
#[derive(Debug, Deserialize)]
struct EventsQuery {
    /// Page number
    #[serde(default = "default_page")]
    page: usize,
    
    /// Page size
    #[serde(default = "default_page_size")]
    page_size: usize,
    
    /// Event type filter
    event_type: Option<String>,
    
    /// Source filter
    source: Option<String>,
}

fn default_page() -> usize {
    0
}

fn default_page_size() -> usize {
    20
}

/// Get events
async fn get_events(
    State(state): State<AppState>,
    Query(query): Query<EventsQuery>,
) -> impl IntoResponse {
    let events = state.events.read().await;
    
    // Apply filters
    let filtered_events: Vec<&Event> = events
        .iter()
        .filter(|e| {
            if let Some(ref event_type) = query.event_type {
                match &e.event_type {
                    EventType::Custom(name) => name == event_type,
                    other => format!("{:?}", other) == event_type,
                }
            } else {
                true
            }
        })
        .filter(|e| {
            if let Some(ref source) = query.source {
                e.source == *source
            } else {
                true
            }
        })
        .collect();
    
    // Paginate
    let total = filtered_events.len();
    let start = query.page * query.page_size;
    let end = (start + query.page_size).min(total);
    
    let paginated = if start < total {
        filtered_events[start..end].to_vec()
    } else {
        Vec::new()
    };
    
    // Convert to response format
    let event_infos: Vec<EventInfo> = paginated
        .into_iter()
        .enumerate()
        .map(|(i, e)| EventInfo {
            id: format!("{}", start + i),
            event_type: match &e.event_type {
                EventType::Custom(name) => name.clone(),
                other => format!("{:?}", other),
            },
            source: e.source.clone(),
            severity: format!("{:?}", e.severity()),
            timestamp: e.timestamp,
            data: e.data.clone(),
        })
        .collect();
    
    let response = EventsResponse {
        events: event_infos,
        total,
        page: query.page,
        page_size: query.page_size,
    };
    
    (StatusCode::OK, Json(response))
}

/// Get current posture
async fn get_posture(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let posture = state.current_posture.read().await.clone();
    
    (StatusCode::OK, Json(serde_json::json!({ "posture": posture })))
}

/// Change posture
async fn change_posture(
    State(state): State<AppState>,
    Json(request): Json<ChangePostureRequest>,
) -> impl IntoResponse {
    let previous_posture;
    
    // Update posture
    {
        let mut posture_lock = state.current_posture.write().await;
        previous_posture = posture_lock.clone();
        *posture_lock = request.posture.clone();
    }
    
    // Send event
    let event = Event::posture_change(
        "pigment_api",
        Some(serde_json::json!({
            "posture": request.posture,
            "previous_posture": previous_posture,
            "source": "api",
        })),
    );
    
    if let Err(e) = state.event_sender.send(event).await {
        tracing::error!("Failed to send posture change event: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Failed to send event" })),
        );
    }
    
    let response = ChangePostureResponse {
        success: true,
        previous_posture,
        new_posture: request.posture,
        timestamp: chrono::Utc::now(),
    };
    
    (StatusCode::OK, Json(response))
}

/// Get active modules
async fn get_modules(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let modules = state.active_modules.read().await.clone();
    
    (StatusCode::OK, Json(modules))
}

/// Toggle module status
async fn toggle_module(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(status): Json<serde_json::Value>,
) -> impl IntoResponse {
    let active = status
        .get("active")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    // Update module status
    {
        let mut modules_lock = state.active_modules.write().await;
        modules_lock.insert(name.clone(), active);
    }
    
    // Send event
    let event = Event::service_lifecycle(
        "pigment_api",
        Some(serde_json::json!({
            "module": name,
            "status": if active { "active" } else { "inactive" },
            "source": "api",
        })),
    );
    
    if let Err(e) = state.event_sender.send(event).await {
        tracing::error!("Failed to send module toggle event: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Failed to send event" })),
        );
    }
    
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "module": name,
            "active": active,
            "timestamp": chrono::Utc::now(),
        })),
    )
}

/// Get system metrics
async fn get_metrics(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let metrics = state.metrics.read().await.clone();
    
    (StatusCode::OK, Json(metrics))
}
