use eframe::egui;
use crate::services::defender::{DefenderService, DefenderStatus};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct ServicesTab {
    defender_status: Option<DefenderStatus>,
    operation_in_progress: bool,
    last_update: std::time::Instant,
    auto_refresh: bool,
}

impl ServicesTab {
    pub fn new() -> Self {
        Self {
            defender_status: None,
            operation_in_progress: false,
            last_update: std::time::Instant::now(),
            auto_refresh: true,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, _app_state: Arc<Mutex<crate::app::AppState>>) {
        ui.heading("🛡️ Gestion des Services Windows");
        ui.separator();

        // Auto-refresh every 3 seconds
        if self.auto_refresh && self.last_update.elapsed().as_secs() >= 3 {
            if let Ok(status) = DefenderService::get_status() {
                self.defender_status = Some(status);
                self.last_update = std::time::Instant::now();
            }
        }

        // Manual refresh button
        ui.horizontal(|ui| {
            if ui.button("🔄 Actualiser le statut").clicked() {
                if let Ok(status) = DefenderService::get_status() {
                    self.defender_status = Some(status);
                    self.last_update = std::time::Instant::now();
                }
            }
            
            ui.checkbox(&mut self.auto_refresh, "Actualisation automatique");
        });

        ui.separator();

        // === DEFENDER STATUS PANEL ===
        egui::CollapsingHeader::new("🛡️ Windows Defender - Statut et Contrôles")
            .default_open(true)
            .show(ui, |ui| {
                self.show_defender_panel(ui);
            });

        ui.separator();

        // === INFORMATION PANEL ===
        egui::CollapsingHeader::new("ℹ️ Informations")
            .default_open(false)
            .show(ui, |ui| {
                ui.label("⚠️ Les modifications de Windows Defender nécessitent des privilèges administrateur.");
                ui.label("🔄 Les changements prennent effet immédiatement sans redémarrage.");
                ui.label("🛡️ La désactivation temporaire est recommandée pour le gaming.");
                ui.label("🔒 Si la Protection contre les Falsifications est active, certaines opérations peuvent échouer.");
            });
    }

    fn show_defender_panel(&mut self, ui: &mut egui::Ui) {
        // Status display
        if let Some(ref status) = self.defender_status {
            // Main status with color coding
            ui.horizontal(|ui| {
                let (color, icon) = if status.real_time_protection {
                    (egui::Color32::from_rgb(46, 125, 50), "🛡️")
                } else {
                    (egui::Color32::from_rgb(198, 40, 40), "❌")
                };

                ui.colored_label(color, format!("{} Statut: {}", icon, status.status_text));
            });

            ui.separator();

            // Detailed status grid
            egui::Grid::new("defender_status_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    for result in &status.last_operation_results {
                        if result.contains("✅") {
                            ui.colored_label(egui::Color32::from_rgb(46, 125, 50), result);
                        } else if result.contains("❌") {
                            ui.colored_label(egui::Color32::from_rgb(198, 40, 40), result);
                        } else if result.contains("⚠️") {
                            ui.colored_label(egui::Color32::from_rgb(255, 152, 0), result);
                        } else if result.contains("🔒") || result.contains("🔓") {
                            ui.colored_label(egui::Color32::from_rgb(63, 81, 181), result);
                        } else if result.contains("☁️") {
                            ui.colored_label(egui::Color32::from_rgb(33, 150, 243), result);
                        } else {
                            ui.label(result);
                        }
                        ui.end_row();
                    }
                });

            ui.separator();

            // Control buttons
            ui.horizontal(|ui| {
                // Disable button
                let disable_button = egui::Button::new("❌ Désactiver Defender")
                    .fill(egui::Color32::from_rgb(198, 40, 40));
                
                if ui.add_enabled(!self.operation_in_progress && status.real_time_protection, disable_button)
                    .on_hover_text("Désactive immédiatement Windows Defender sans redémarrage")
                    .clicked() 
                {
                    self.operation_in_progress = true;
                    self.perform_defender_operation(false);
                }

                // Enable button  
                let enable_button = egui::Button::new("✅ Activer Defender")
                    .fill(egui::Color32::from_rgb(46, 125, 50));
                
                if ui.add_enabled(!self.operation_in_progress && !status.real_time_protection, enable_button)
                    .on_hover_text("Réactive immédiatement Windows Defender")
                    .clicked() 
                {
                    self.operation_in_progress = true;
                    self.perform_defender_operation(true);
                }

                if self.operation_in_progress {
                    ui.spinner();
                    ui.label("Opération en cours...");
                }
            });

        } else {
            // Initial loading state
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Chargement du statut de Windows Defender...");
            });

            // Try to load status
            if let Ok(status) = DefenderService::get_status() {
                self.defender_status = Some(status);
            }
        }

        ui.separator();

        // Real-time monitoring indicators
        if let Some(ref status) = self.defender_status {
            ui.horizontal(|ui| {
                ui.label("Surveillance en temps réel:");
                
                let protection_color = if status.real_time_protection {
                    egui::Color32::from_rgb(46, 125, 50)
                } else {
                    egui::Color32::from_rgb(158, 158, 158)
                };
                
                ui.colored_label(protection_color, "●");
                ui.label("Protection");

                ui.separator();

                let tamper_color = if status.tamper_protection {
                    egui::Color32::from_rgb(255, 152, 0)
                } else {
                    egui::Color32::from_rgb(158, 158, 158)
                };
                
                ui.colored_label(tamper_color, "●");
                ui.label("Tamper");

                ui.separator();

                let cloud_color = if status.cloud_protection {
                    egui::Color32::from_rgb(33, 150, 243)
                } else {
                    egui::Color32::from_rgb(158, 158, 158)
                };
                
                ui.colored_label(cloud_color, "●");
                ui.label("Cloud");
            });
        }
    }

    fn perform_defender_operation(&mut self, enable: bool) {
        // This would normally be async, but for simplicity we'll do it synchronously
        // In a real application, you'd want to use tokio::spawn or similar
        
        let result = if enable {
            DefenderService::enable_immediately()
        } else {
            DefenderService::disable_immediately()
        };

        match result {
            Ok(status) => {
                self.defender_status = Some(status);
            }
            Err(e) => {
                // Create error status
                self.defender_status = Some(DefenderStatus {
                    status_text: format!("❌ Erreur: {}", e),
                    last_operation_results: vec![
                        format!("Échec de l'opération: {}", e),
                        "Vérifiez que vous avez les privilèges administrateur".to_string(),
                        "La Protection contre les Falsifications peut bloquer l'opération".to_string(),
                    ],
                    ..Default::default()
                });
            }
        }

        self.operation_in_progress = false;
    }
} 