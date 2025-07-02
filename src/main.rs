#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod disk;
mod memory;
mod network;
mod os_info;
mod scheduler;
mod services;
mod theme;
mod ui;
mod utils;

use egui::IconData;
use image::io::Reader as ImageReader;
use std::io::Cursor;
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use ui::app::CleanRamApp;

fn main() {
    let _guard = setup_logging();

    info!("🚀 Initializing GameBooster application...");

    // Check for startup arguments
    let (is_minimized, should_auto_start_scheduler) = crate::scheduler::TaskScheduler::check_startup_args();
    
    if is_minimized {
        info!("🔽 Starting in minimized mode (Windows startup)");
    }
    
    if should_auto_start_scheduler {
        info!("⏰ Auto-starting scheduler as configured");
    }

    // Automatic QoS system test at startup (release mode only)
    #[cfg(not(debug_assertions))]
    test_qos_system();

    let icon = load_icon().expect("Failed to load application icon.");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([900.0, 500.0])
            .with_title("GameBooster - Network QoS Ready")
            .with_icon(Arc::new(icon)) // Set the window icon
            .with_resizable(true),
        centered: true,
        ..Default::default()
    };

    info!("Starting eframe::run_native...");
    
    if let Err(e) = eframe::run_native(
        "GameBooster",
        native_options,
        Box::new(move |cc| {
            let mut app = CleanRamApp::new(cc);
            
            // Auto-start scheduler if configured
            if should_auto_start_scheduler {
                app.scheduler_running = true;
                info!("✅ Scheduler auto-started");
            }
            
            Box::new(app)
        }),
    ) {
        eprintln!("Error running eframe: {}", e);
    }
}

/// Loads the application icon from the assets folder.
fn load_icon() -> Result<IconData, Box<dyn std::error::Error>> {
    let bytes = include_bytes!("../assets/img/logo.png");
    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;
    let image_buffer = image.to_rgba8();
    Ok(IconData {
        rgba: image_buffer.into_raw(),
        width: image.width(),
        height: image.height(),
    })
}

/// Automatic QoS system test at startup
#[cfg(not(debug_assertions))]
fn test_qos_system() {
    use crate::network::NetworkLimiter;
    
    info!("🧪 Automatic QoS system test...");
    
    match NetworkLimiter::new() {
        Ok(mut limiter) => {
            info!("✅ NetworkLimiter created");
            
            match limiter.scan_network_processes() {
                Ok(()) => {
                    let processes = limiter.get_processes();
                    info!("✅ Network scan: {} processes detected", processes.len());
                    
                    // Display some detected processes
                    for (i, process) in processes.iter().take(3).enumerate() {
                        info!("  {}. {} (PID: {}) - {}↓ {}↑", 
                            i + 1, 
                            process.name, 
                            process.pid,
                            crate::network::format_speed(process.current_download_speed),
                            crate::network::format_speed(process.current_upload_speed)
                        );
                    }
                    
                    // Test existing policies verification
                    match limiter.verify_qos_policies() {
                        Ok(policies) => {
                            if policies.is_empty() {
                                info!("📋 No active GameBooster QoS policy");
                            } else {
                                info!("📋 {} active GameBooster QoS policies", policies.len());
                            }
                        }
                        Err(e) => {
                            warn!("⚠️ Unable to verify QoS policies: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️ Network scan error: {}", e);
                }
            }
        }
        Err(e) => {
            warn!("⚠️ Unable to create NetworkLimiter: {}", e);
        }
    }
    
    info!("🎯 QoS system ready for use");
}

fn setup_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    // Create logs directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all("logs") {
        eprintln!("Failed to create logs directory: {}", e);
        return None;
    }

    // File appender for logs
    let file_appender = tracing_appender::rolling::daily("logs", "gamebooster.log");
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    // Console writer
    let (non_blocking_stdout, _guard_stdout) = tracing_appender::non_blocking(std::io::stdout());

    // Build subscriber with both file and console outputs
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking_file)
                .with_ansi(false)
                .with_target(false)
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking_stdout)
                .with_ansi(true)
                .with_target(false)
        )
        .with(EnvFilter::new("info"));

    // Set the subscriber as the global default
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Failed to set tracing subscriber: {}", e);
        return None;
    }

    Some(guard)
}