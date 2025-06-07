#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod disk;
mod memory;
mod network;
mod os_info;
mod scheduler;
mod services;
mod theme;
mod ui;

use crate::ui::app::CleanRamApp;
use eframe::egui;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const LOGO_BYTES: &[u8] = include_bytes!("../img/logo.png");
const RAM_ICON_BYTES: &[u8] = include_bytes!("../img/ram.png");

fn main() {
    let _guard = setup_logging();

    info!("Initializing GameBooster application...");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    info!("Starting eframe::run_native...");
    eframe::run_native(
        "GameBooster",
        native_options,
        Box::new(|cc| Box::new(CleanRamApp::new(cc))),
    )
    .expect("eframe::run_native failed");
    info!("eframe::run_native finished. Application is closing.");
}

fn setup_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    if std::fs::create_dir_all("logs").is_ok() {
        let file_appender = tracing_appender::rolling::daily("logs", "app.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
            .init();
        Some(guard)
    } else {
        // Fallback to console if directory creation fails
        tracing_subscriber::fmt::init();
        None
    }
}