use eframe::egui;

pub fn services_ui(app: &mut crate::CleanRamApp, ui: &mut egui::Ui) {
    ui.heading("🛡️ DÉSACTIVATION WINDOWS DEFENDER - IMMEDIAT");
    ui.separator();

    // Test immédiat du statut - STOCKE le résultat dans l'app
    if ui.button("🔍 VÉRIFIER STATUT DEFENDER").clicked() {
        match crate::services::defender::DefenderService::get_status() {
            Ok(status) => {
                app.last_defender_status = Some(Ok(status));
            }
            Err(e) => {
                app.last_defender_status = Some(Err(e));
            }
        }
    }

    // Affiche le statut stocké si disponible
    if let Some(ref status_result) = app.last_defender_status {
        match status_result {
            Ok(status) => {
                if status.real_time_protection {
                    ui.colored_label(egui::Color32::RED, "❌ DEFENDER EST ACTIF");
                } else {
                    ui.colored_label(egui::Color32::GREEN, "✅ DEFENDER EST DÉSACTIVÉ");
                }
            }
            Err(e) => {
                ui.colored_label(egui::Color32::YELLOW, format!("⚠️ Erreur: {}", e));
            }
        }
    }

    ui.separator();

    // BOUTON DÉSACTIVATION IMMÉDIATE
    if ui.button("❌ DÉSACTIVER DEFENDER MAINTENANT").clicked() {
        match crate::services::defender::DefenderService::disable_immediately() {
            Ok(result) => {
                ui.colored_label(egui::Color32::GREEN, "✅ DÉSACTIVATION LANCÉE !");
                for res in result.last_operation_results {
                    ui.label(res);
                }
            }
            Err(e) => {
                ui.colored_label(egui::Color32::RED, format!("❌ ERREUR: {}", e));
            }
        }
    }

    // BOUTON RÉACTIVATION
    if ui.button("✅ RÉACTIVER DEFENDER").clicked() {
        match crate::services::defender::DefenderService::enable_immediately() {
            Ok(result) => {
                ui.colored_label(egui::Color32::GREEN, "✅ RÉACTIVATION LANCÉE !");
                for res in result.last_operation_results {
                    ui.label(res);
                }
            }
            Err(e) => {
                ui.colored_label(egui::Color32::RED, format!("❌ ERREUR: {}", e));
            }
        }
    }

    ui.separator();
    ui.label("⚡ Les changements prennent effet IMMÉDIATEMENT sans redémarrage !");

    ui.separator();

    // === DEFENDER CONTROL PANEL ===
    egui::CollapsingHeader::new("🛡️ Windows Defender - Contrôle Immédiat")
        .default_open(true)
        .show(ui, |ui| {
            ui.label("⚡ Désactivation/Activation IMMÉDIATE sans redémarrage");
            ui.separator();

            // Status check
            let defender_status = crate::services::defender::DefenderService::get_status().unwrap_or_default();
            
            // Display current status
            ui.horizontal(|ui| {
                let (color, icon) = if defender_status.real_time_protection {
                    (egui::Color32::from_rgb(46, 125, 50), "🛡️ ACTIF")
                } else {
                    (egui::Color32::from_rgb(198, 40, 40), "❌ DÉSACTIVÉ")
                };
                
                ui.colored_label(color, format!("Statut: {}", icon));
            });

            ui.separator();

            // Detailed status
            ui.label("📊 Détails de protection:");
            egui::Grid::new("defender_details")
                .num_columns(2)
                .show(ui, |ui| {
                    for detail in &defender_status.last_operation_results {
                        if detail.contains("✅") {
                            ui.colored_label(egui::Color32::from_rgb(46, 125, 50), detail);
                        } else if detail.contains("❌") {
                            ui.colored_label(egui::Color32::from_rgb(198, 40, 40), detail);
                        } else if detail.contains("🔒") || detail.contains("🔓") {
                            ui.colored_label(egui::Color32::from_rgb(63, 81, 181), detail);
                        } else {
                            ui.label(detail);
                        }
                        ui.end_row();
                    }
                });
        });

    ui.separator();

    // === INFORMATION PANEL ===
    egui::CollapsingHeader::new("ℹ️ Informations Importantes")
        .default_open(false)
        .show(ui, |ui| {
            ui.colored_label(egui::Color32::YELLOW, "⚠️ ATTENTION :");
            ui.label("• Les modifications prennent effet IMMÉDIATEMENT");
            ui.label("• Aucun redémarrage nécessaire");
            ui.label("• Privilèges administrateur requis");
            ui.label("• La Protection contre les Falsifications peut bloquer certaines opérations");
            ui.separator();
            ui.colored_label(egui::Color32::from_rgb(33, 150, 243), "🎮 Pour le Gaming :");
            ui.label("• Désactivation temporaire recommandée");
            ui.label("• Réactivation après session de jeu");
        });
}