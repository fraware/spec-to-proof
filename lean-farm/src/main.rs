use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use tokio::signal;
use tracing::{info, warn, error};
use clap::Parser;

use lean_farm::config::Config;
use lean_farm::job_runner::JobRunner;
use lean_farm::security::SecurityManager;
use lean_farm::metrics::MetricsServer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "/etc/lean-farm/config.yaml")]
    config: PathBuf,
    
    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
    
    /// Log format (json, text)
    #[arg(long, default_value = "json")]
    log_format: String,
    
    /// Metrics port
    #[arg(long, default_value = "9090")]
    metrics_port: u16,
    
    /// Metrics path
    #[arg(long, default_value = "/metrics")]
    metrics_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    // Initialize logging
    init_logging(&args.log_level, &args.log_format)?;
    
    info!("Starting Lean Farm Job Runner");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Build timestamp: {}", env!("VERGEN_BUILD_TIMESTAMP"));
    
    // Load configuration
    let config = Config::from_file(&args.config).await?;
    info!("Configuration loaded from {:?}", args.config);
    
    // Initialize security manager
    let security_manager = SecurityManager::new(&config.security)?;
    info!("Security manager initialized");
    
    // Validate security requirements
    security_manager.validate_environment().await?;
    info!("Security validation passed");
    
    // Initialize metrics server
    let metrics_server = MetricsServer::new(args.metrics_port, args.metrics_path);
    let metrics_handle = tokio::spawn(metrics_server.start());
    info!("Metrics server started on port {}", args.metrics_port);
    
    // Initialize job runner
    let job_runner = JobRunner::new(config, security_manager).await?;
    info!("Job runner initialized");
    
    // Start health check server
    let health_handle = tokio::spawn(job_runner.start_health_server());
    info!("Health check server started");
    
    // Start job processing
    let job_handle = tokio::spawn(job_runner.start_processing());
    info!("Job processing started");
    
    // Wait for shutdown signal
    wait_for_shutdown().await;
    
    info!("Shutting down Lean Farm Job Runner");
    
    // Graceful shutdown
    job_handle.abort();
    health_handle.abort();
    metrics_handle.abort();
    
    info!("Lean Farm Job Runner stopped");
    Ok(())
}

fn init_logging(level: &str, format: &str) -> Result<(), Box<dyn Error>> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| level.parse().unwrap());
    
    let subscriber = tracing_subscriber::Subscriber::builder()
        .with_env_filter(env_filter);
    
    match format {
        "json" => {
            let subscriber = subscriber
                .with_ansi(false)
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .json()
                .with_current_span(true)
                .with_span_list(true)
                .with_max_level(tracing::Level::TRACE)
                .finish();
            tracing::subscriber::set_global_default(subscriber)?;
        }
        "text" => {
            let subscriber = subscriber
                .with_ansi(true)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .finish();
            tracing::subscriber::set_global_default(subscriber)?;
        }
        _ => {
            return Err("Unsupported log format. Use 'json' or 'text'".into());
        }
    }
    
    Ok(())
}

async fn wait_for_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
    };
    
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to listen for SIGTERM")
            .recv()
            .await;
    };
    
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    
    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating shutdown");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating shutdown");
        }
    }
} 