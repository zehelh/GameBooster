use crate::theme::{self};
use crate::ui::app::CleanRamApp;
use eframe::egui;

pub fn draw_settings_tab(app: &mut CleanRamApp, ui: &mut egui::Ui) {
    ui.heading("Paramètres");

    ui.add_space(20.0);

    // --- Theme Selection ---
    ui.group(|ui| {
        ui.label("Thème de l'application");
        ui.horizontal(|ui| {
            if ui.selectable_label(app.theme.name == "Light", "Clair").clicked() {
                app.theme = theme::light_theme();
                ui.ctx().set_visuals(app.theme.visuals.clone());
            }
            if ui.selectable_label(app.theme.name == "Dark", "Sombre").clicked() {
                app.theme = theme::dark_theme();
                ui.ctx().set_visuals(app.theme.visuals.clone());
            }
        });
    });
    
    ui.add_space(20.0);

    // --- System Information ---
    ui.group(|ui| {
        ui.label("Informations Système");
        ui.separator();
        
        // Use the already determined version string from the app state
        ui.label(format!("Version de Windows : {}", app.windows_version_string));
        
        // You can add more system info here if needed
        // For example: CPU, GPU, RAM size, etc.
    });
    
    ui.add_space(20.0);
    
    // --- About Section ---
    ui.group(|ui| {
        ui.label("À propos");
        ui.separator();
        ui.label(format!("GameBooster v{}", env!("CARGO_PKG_VERSION")));
        ui.horizontal(|ui| {
            ui.label("Créé avec");
            ui.hyperlink_to("egui", "https://github.com/emilk/egui");
            ui.label("et Rust.");
        });
    });
}