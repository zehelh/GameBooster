use anyhow::Result;
use eframe::egui::{self, Vec2};
use crate::ui::icons::create_app_icon;
use crate::ui::app::CleanRamApp as UiCleanRamApp;

// Importation des modules
mod disk;
mod memory;
mod scheduler;
mod services;
mod ui;
mod utils;

// Logos intégrés en tant que ressources
const LOGO_BYTES: &[u8] = include_bytes!("../img/logo.png");
const RAM_ICON_BYTES: &[u8] = include_bytes!("../img/ram.png");

fn main() -> Result<(), eframe::Error> {
    // Vérification des droits administrateur
    let is_admin = is_elevated::is_elevated();
    if !is_admin {
        println!("Attention: L'application fonctionne mieux avec des droits administrateur.");
        // Continuer quand même, l'interface affichera un avertissement
    }
    
    // Options de l'application
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(500.0, 600.0)) // Adjusted for potentially less content
            .with_min_inner_size(Vec2::new(400.0, 300.0))
            .with_icon(create_app_icon(LOGO_BYTES)),
        centered: true,
        default_theme: eframe::Theme::Dark,
        follow_system_theme: false,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        vsync: true,
        ..Default::default()
    };
      // Démarrer l'application
    eframe::run_native(
        "GameBooster - Optimiseur PC Gaming",
        options,
        Box::new(|cc| {
            // Utiliser la version du module ui::app
            Box::new(UiCleanRamApp::new(cc, LOGO_BYTES, RAM_ICON_BYTES))
        }),
    )
}