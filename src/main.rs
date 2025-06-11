#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod disk;
mod memory;
mod network;
mod os_info;
mod scheduler;
mod services;
mod theme;
mod ui;

use ui::app::CleanRamApp;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

fn main() {
    let _guard = setup_logging();

    info!("🚀 Initializing GameBooster application...");

    // Test QoS automatique au démarrage (mode release uniquement)
    #[cfg(not(debug_assertions))]
    test_qos_system();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([900.0, 500.0])
            .with_title("GameBooster - Network QoS Ready")
            .with_resizable(true),
        centered: true,
        ..Default::default()
    };

    info!("Starting eframe::run_native...");
    
    if let Err(e) = eframe::run_native(
        "GameBooster",
        native_options,
        Box::new(|cc| {
            let app = CleanRamApp::new(cc);
            Box::new(app)
        }),
    ) {
        eprintln!("Error running eframe: {}", e);
    }
}

/// Test automatique du système QoS au démarrage
#[cfg(not(debug_assertions))]
fn test_qos_system() {
    use crate::network::NetworkLimiter;
    
    info!("🧪 Test système QoS automatique...");
    
    match NetworkLimiter::new() {
        Ok(mut limiter) => {
            info!("✅ NetworkLimiter créé");
            
            match limiter.scan_network_processes() {
                Ok(()) => {
                    let processes = limiter.get_processes();
                    info!("✅ Scan réseau: {} processus détectés", processes.len());
                    
                    // Afficher quelques processus détectés
                    for (i, process) in processes.iter().take(3).enumerate() {
                        info!("  {}. {} (PID: {}) - {}↓ {}↑", 
                            i + 1, 
                            process.name, 
                            process.pid,
                            crate::network::format_speed(process.current_download_speed),
                            crate::network::format_speed(process.current_upload_speed)
                        );
                    }
                    
                    // Test vérification politiques existantes
                    match limiter.verify_qos_policies() {
                        Ok(policies) => {
                            if policies.is_empty() {
                                info!("📋 Aucune politique QoS GameBooster active");
                            } else {
                                info!("📋 {} politiques QoS GameBooster actives", policies.len());
                            }
                        }
                        Err(e) => {
                            warn!("⚠️ Impossible de vérifier les politiques QoS: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️ Erreur scan réseau: {}", e);
                }
            }
        }
        Err(e) => {
            warn!("⚠️ Impossible de créer NetworkLimiter: {}", e);
        }
    }
    
    info!("🎯 Système QoS prêt pour utilisation");
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