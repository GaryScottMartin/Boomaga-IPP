//! IPP backend service for boomaga virtual printer
//!
//! This service implements an IPP (Internet Printing Protocol) server
//! that receives print jobs and manages the print queue.

mod server;
mod job_processor;
mod job_queue;

use tracing::{info, error, warn, Level};
use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Initialize logging
    let log_level = if args.contains(&"--debug".to_string()) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    info!("{} v{} starting...", boomaga_core::constants::APP_NAME, boomaga_core::constants::APP_VERSION);

    // Parse configuration
    let config = parse_config(&args)?;

    info!("Configuration loaded:");
    info!("  - IPC socket: {:?}", config.ipc_socket_path);
    info!("  - D-Bus service: {}", config.dbus_service_name);
    info!("  - IPP port: {}", config.ipp_port);

    // Create job queue
    let job_queue = job_queue::JobQueue::new(config.job_queue_size)?;

    // Start job processor
    let processor = job_processor::JobProcessor::new(job_queue, config.max_concurrent_jobs, config.worker_threads)?;

    // Start IPP server
    let mut ipp_server = server::IppServer::new(
        config.ipp_port,
        config.ipc_socket_path,
        config.dbus_service_name,
        processor.clone(),
    );

    info!("Starting IPP server on port {}", config.ipp_port);

    // Start server
    if let Err(e) = ipp_server.run() {
        error!("IPP server error: {}", e);
        return Err(e);
    }

    Ok(())
}

/// Application configuration
struct AppConfig {
    ipc_socket_path: PathBuf,
    dbus_service_name: String,
    ipp_port: u16,
    max_concurrent_jobs: usize,
    worker_threads: usize,
    job_queue_size: usize,
}

/// Parse command line arguments and configuration
fn parse_config(args: &[String]) -> anyhow::Result<AppConfig> {
    let mut ipc_socket_path = std::path::PathBuf::from(boomaga_core::constants::IPC_SOCKET_PATH);
    let mut dbus_service_name = boomaga_core::constants::DBUS_SERVICE_NAME.to_string();
    let mut ipp_port = boomaga_core::constants::IPP_PORT;
    let mut max_concurrent_jobs = boomaga_core::constants::MAX_CONCURRENT_JOBS;
    let mut worker_threads = boomaga_core::constants::WORKER_THREADS;
    let mut job_queue_size = boomaga_core::constants::JOB_QUEUE_SIZE;

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--socket" => {
                if i + 1 < args.len() {
                    ipc_socket_path = args[i + 1].clone().into();
                    i += 2;
                } else {
                    anyhow::bail!("--socket requires a path argument");
                }
            }
            "--dbus" => {
                if i + 1 < args.len() {
                    dbus_service_name = args[i + 1].clone();
                    i += 2;
                } else {
                    anyhow::bail!("--dbus requires a service name argument");
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    ipp_port = args[i + 1].parse().unwrap_or(631);
                    i += 2;
                } else {
                    anyhow::bail!("--port requires a port number argument");
                }
            }
            "--concurrent" => {
                if i + 1 < args.len() {
                    max_concurrent_jobs = args[i + 1].parse().unwrap_or(4);
                    i += 2;
                } else {
                    anyhow::bail!("--concurrent requires a number argument");
                }
            }
            "--workers" => {
                if i + 1 < args.len() {
                    worker_threads = args[i + 1].parse().unwrap_or(2);
                    i += 2;
                } else {
                    anyhow::bail!("--workers requires a number argument");
                }
            }
            "--queue-size" => {
                if i + 1 < args.len() {
                    job_queue_size = args[i + 1].parse().unwrap_or(100);
                    i += 2;
                } else {
                    anyhow::bail!("--queue-size requires a number argument");
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                warn!("Unknown argument: {}", args[i]);
                i += 1;
            }
        }
    }

    Ok(AppConfig {
        ipc_socket_path,
        dbus_service_name,
        ipp_port,
        max_concurrent_jobs,
        worker_threads,
        job_queue_size,
    })
}

/// Print usage help
fn print_help() {
    println!("{} v{} - IPP Backend Service", boomaga_core::constants::APP_NAME, boomaga_core::constants::APP_VERSION);
    println!();
    println!("Usage: boomaga-ipp-backend [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --socket <path>        Set IPC socket path (default: {})", boomaga_core::constants::IPC_SOCKET_PATH);
    println!("  --dbus <name>          Set D-Bus service name (default: {})", boomaga_core::constants::DBUS_SERVICE_NAME);
    println!("  --port <number>        Set IPP port (default: {})", boomaga_core::constants::IPP_PORT);
    println!("  --concurrent <number>  Maximum concurrent jobs (default: {})", boomaga_core::constants::MAX_CONCURRENT_JOBS);
    println!("  --workers <number>     Number of worker threads (default: {})", boomaga_core::constants::WORKER_THREADS);
    println!("  --queue-size <number>  Job queue size (default: {})", boomaga_core::constants::JOB_QUEUE_SIZE);
    println!("  --debug                 Enable debug logging");
    println!("  --help, -h              Show this help message");
    println!();
    println!("Example:");
    println!("  boomaga-ipp-backend --socket /tmp/boomaga.sock --port 631");
    println!();
    println!("{}", boomaga_core::constants::APP_DESCRIPTION);
}
