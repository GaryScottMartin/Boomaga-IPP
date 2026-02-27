//! Preview application for boomaga virtual printer
//!
//! This application provides a document previewer with native Wayland GUI
//! using the Druid framework.

mod app;
mod viewer;
mod document_view;
mod menu_bar;
mod toolbar;
mod print_dialog;
mod settings_dialog;

use tracing::{info, error, Level};
use std::env;

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
    info!("  - Preview window: {:?}", config.preview_window);

    // Create and run application
    let options = druid::WindowDesc::new()
        .title(boomaga_core::constants::APP_NAME)
        .content(app::BoomagaApp::new().into());

    // Run the application
    if let Err(e) = druid::run_app(options) {
        error!("Application error: {}", e);
        return Err(e);
    }

    Ok(())
}

/// Application configuration
struct AppConfig {
    preview_window: std::path::PathBuf,
}

/// Parse command line arguments and configuration
fn parse_config(args: &[String]) -> anyhow::Result<AppConfig> {
    let mut preview_window = std::path::PathBuf::new();

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--window" | "-w" => {
                if i + 1 < args.len() {
                    preview_window = args[i + 1].clone().into();
                    i += 2;
                } else {
                    anyhow::bail!("--window requires a path argument");
                }
            }
            "--debug" => {
                // Debug flag already handled
                i += 1;
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

    Ok(AppConfig { preview_window })
}

/// Print usage help
fn print_help() {
    println!("{} v{} - Preview Application", boomaga_core::constants::APP_NAME, boomaga_core::constants::APP_VERSION);
    println!();
    println!("Usage: boomaga-preview [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --window <path>        Set preview window size (default: 800x600)");
    println!("  --debug                 Enable debug logging");
    println!("  --help, -h              Show this help message");
    println!();
    println!("Example:");
    println!("  boomaga-preview --window 1200x900");
    println!();
    println!("{} {}", boomaga_core::constants::APP_NAME, boomaga_core::constants::APP_DESCRIPTION);
}
