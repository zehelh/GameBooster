use crate::memory::{clean_memory, get_detailed_system_memory_info, CleaningResults};
use crate::theme::Theme;
use crate::ui::app::CleanRamApp;
use eframe::egui::{self, Layout, RichText, ProgressBar};
use poll_promise::Promise;

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
}

pub fn draw_memory_tab(app: &mut CleanRamApp, ui: &mut egui::Ui, _theme: &Theme) {
    let mem_info = get_detailed_system_memory_info();

    // Mettre à jour l'utilisation de la RAM uniquement si aucune opération de nettoyage n'est en cours
    if app.cleaning_promise.is_none() {
        app.ram_usage = mem_info.used_physical_percent();
    }

    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.heading("Optimisation de la Mémoire");
        ui.add_space(10.0);
    });
    
    ui.separator();
    ui.add_space(10.0);

    // --- Physical Memory Section ---
    ui.group(|ui| {
        ui.heading("Mémoire Physique (RAM)");
        ui.add_space(5.0);

        let used_gb = bytes_to_gb(mem_info.used_physical());
        let total_gb = bytes_to_gb(mem_info.total_physical);
        let usage_percent = mem_info.used_physical_percent() / 100.0;
        
        ui.label(format!("Utilisation : {:.2} GB / {:.2} GB", used_gb, total_gb));

        let progress_bar = ProgressBar::new(usage_percent)
            .show_percentage()
            .text(format!("{:.1} %", usage_percent * 100.0));
        ui.add(progress_bar);
    });

    ui.add_space(10.0);

    // --- Pagefile Section ---
    ui.group(|ui| {
        ui.heading("Fichier d'échange (Mémoire Virtuelle)");
        ui.add_space(5.0);

        let used_pagefile = mem_info.total_pagefile - mem_info.avail_pagefile;
        let used_gb = bytes_to_gb(used_pagefile);
        let total_gb = bytes_to_gb(mem_info.total_pagefile);
        let usage_percent = if mem_info.total_pagefile > 0 {
            used_pagefile as f32 / mem_info.total_pagefile as f32
        } else {
            0.0
        };

        ui.label(format!("Utilisation : {:.2} GB / {:.2} GB", used_gb, total_gb));

        let progress_bar = ProgressBar::new(usage_percent)
            .show_percentage()
            .text(format!("{:.1} %", usage_percent * 100.0));
        ui.add(progress_bar);
    });

    ui.add_space(20.0);

    // --- Clean Button ---
    ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
        let button_size = egui::vec2(200.0, 40.0);
        let clean_button = egui::Button::new("Nettoyer la RAM").min_size(button_size);

        let is_cleaning = app.cleaning_promise.is_some();
        ui.add_enabled(!is_cleaning, clean_button).on_hover_text("Nettoie les processus et le working set de l'application.")
            .clicked().then(|| {
                let promise = Promise::spawn_thread("memory_clean", || {
                    // Gérer le Result de clean_memory
                    match clean_memory() {
                        Ok(results) => results,
                        Err(e) => {
                            // En cas d'erreur, créer un CleaningResults avec le message d'erreur
                            let mut error_results = CleaningResults::new();
                            error_results.has_error = true;
                            error_results.error_message = format!("Erreur lors du nettoyage de la mémoire : {}", e);
                            error_results.is_completed = true;
                            error_results.end_time = Some(chrono::Local::now());
                            error_results
                        }
                    }
                });
                app.cleaning_promise = Some(promise);
            });

        if is_cleaning {
            ui.spinner();
            ui.ctx().request_repaint(); // Keep repainting while cleaning
        }
    });


    if let Some(promise) = &app.cleaning_promise {
        if let Some(results) = promise.ready() {
            app.last_cleaned_results = Some(results.clone());
            app.cleaning_promise = None;
            // No need to manually update ram_usage here, it will be updated on the next frame
        }
    }
    
    if let Some(results) = &app.last_cleaned_results {
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Résultat du Nettoyage").strong());
        });

        if results.has_error {
            ui.colored_label(egui::Color32::RED, &results.error_message);
        } else {
            let freed_mb = results.total_freed() as f64 / 1024.0 / 1024.0;
            if freed_mb > 0.0 || !results.processes.is_empty() {
                ui.label(format!("Mémoire libérée : {:.2} MB", freed_mb));
                ui.label(format!("Processus optimisés : {}", results.processes.len()));
            } else {
                // Afficher le message spécifique de Linux si aucune mémoire n'a été "libérée"
                // et qu'aucun processus n'a été listé.
                ui.label(&results.error_message); 
            }

            if !results.processes.is_empty() {
                ui.add_space(10.0);
                egui::CollapsingHeader::new("Détails de l'optimisation").show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for process in &results.processes {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}", process.name));
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    let mem_freed_mb = process.memory_freed as f64 / 1024.0 / 1024.0;
                                    ui.label(format!("{:.2} MB", mem_freed_mb));
                                });
                            });
                        }
                    });
                });
            }
        }
    }
}