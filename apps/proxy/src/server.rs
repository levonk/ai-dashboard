use anyhow::{Context, Result};
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::compression::CompressionLayer;
use tracing::info;

use crate::monitoring::HardwareMonitor;
use crate::collection::{SystemMetricsCollector, MetricsScheduler};

// use api::health_check;
// pub mod api;

pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub enable_system_metrics: bool,
    pub system_metrics_interval_seconds: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
            enable_system_metrics: true,
            system_metrics_interval_seconds: 1,
        }
    }
}

pub struct ServerState {
    pub hardware_monitor: Arc<HardwareMonitor>,
    pub system_metrics_collector: Option<Arc<SystemMetricsCollector>>,
    pub metrics_scheduler: Option<Arc<MetricsScheduler>>,
}

impl ServerState {
    pub fn new(config: &ServerConfig) -> Result<Self> {
        info!("Initializing server state");
        
        // Initialize hardware monitor
        let hardware_monitor = Arc::new(HardwareMonitor::new()
            .context("Failed to initialize hardware monitor")?);
        
        // Initialize system metrics collector if enabled
        let (system_metrics_collector, metrics_scheduler) = if config.enable_system_metrics {
            info!("System metrics collection enabled with {}s interval", config.system_metrics_interval_seconds);
            
            let collector = Arc::new(SystemMetricsCollector::new(
                Arc::clone(&hardware_monitor)
            ));
            
            let scheduler = Arc::new(MetricsScheduler::new(
                Arc::clone(&collector),
                config.system_metrics_interval_seconds
            ));
            
            (Some(collector), Some(scheduler))
        } else {
            info!("System metrics collection disabled");
            (None, None)
        };
        
        Ok(ServerState {
            hardware_monitor,
            system_metrics_collector,
            metrics_scheduler,
        })
    }
    
    pub async fn start_metrics_collection(&self) -> Result<()> {
        if let Some(scheduler) = &self.metrics_scheduler {
            info!("Starting system metrics collection scheduler");
            scheduler.start().await
                .context("Failed to start metrics scheduler")?;
        }
        Ok(())
    }
    
    pub async fn stop_metrics_collection(&self) -> Result<()> {
        if let Some(scheduler) = &self.metrics_scheduler {
            info!("Stopping system metrics collection scheduler");
            scheduler.stop().await
                .context("Failed to stop metrics scheduler")?;
        }
        Ok(())
    }
}

pub async fn run_server(config: ServerConfig) -> Result<()> {
    let addr = format!("{}:{}", config.host, config.port);
    let socket_addr: SocketAddr = addr.parse()
        .context("Invalid server address")?;

    // Initialize server state
    let state = ServerState::new(&config)
        .context("Failed to initialize server state")?;
    
    // Start metrics collection if enabled
    state.start_metrics_collection().await?;

    // Build the application router
    let app = Router::new()
        // .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

    let listener = TcpListener::bind(socket_addr)
        .await
        .context("Failed to bind to address")?;

    info!("Server starting on http://{}", socket_addr);
    
    // Run the server
    let server_result = axum::serve(listener, app)
        .await
        .context("Server error");
    
    // Stop metrics collection on shutdown
    state.stop_metrics_collection().await?;
    
    server_result?;

    Ok(())
}
