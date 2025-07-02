use crate::theme::{self};
use crate::ui::app::CleanRamApp;
use crate::os_info::get_windows_version_string;
use crate::scheduler::TaskScheduler;
use eframe::egui;

pub fn draw_settings_tab(app: &mut CleanRamApp, ui: &mut egui::Ui) {
    ui.heading("Paramètres");

    ui.add_space(20.0);

    // --- Auto-Startup Settings ---
    ui.group(|ui| {
        ui.label("🚀 Démarrage automatique");
        ui.separator();
        
        let mut config = TaskScheduler::load_startup_config();
        let mut config_changed = false;

        ui.horizontal(|ui| {
            if ui.checkbox(&mut config.enabled, "Démarrer avec Windows").changed() {
                config_changed = true;
            }
            
            if config.enabled {
                if ui.checkbox(&mut config.start_minimized, "Démarrer en mode réduit").changed() {
                    config_changed = true;
                }
                
                if ui.checkbox(&mut config.auto_start_scheduler, "Démarrer le planificateur automatiquement").changed() {
                    config_changed = true;
                }
            }
        });

        if config.enabled {
            ui.horizontal(|ui| {
                ui.label("Délai de démarrage (secondes):");
                if ui.add(egui::Slider::new(&mut config.startup_delay_seconds, 0..=300)).changed() {
                    config_changed = true;
                }
            });

            ui.small("Le délai aide à éviter les conflits avec les processus de démarrage du système.");
        }

        if config_changed {
            if let Err(e) = TaskScheduler::save_startup_config(&config) {
                tracing::error!("Failed to save startup config: {}", e);
            }

            // Apply Windows registry changes
            #[cfg(target_os = "windows")]
            if let Err(e) = TaskScheduler::set_windows_startup(config.enabled, config.start_minimized) {
                tracing::error!("Failed to update Windows startup registry: {}", e);
            }
        }
    });

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
        
        let version_string = get_windows_version_string();
        ui.label(format!("Version de Windows : {}", version_string));
        
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