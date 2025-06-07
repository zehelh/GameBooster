use eframe::egui;
use egui::ProgressBar;
use crate::ui::app::CleanRamApp;
use poll_promise::Promise;

pub fn draw_disk_tab(app: &mut CleanRamApp, ui: &mut egui::Ui) {
    ui.heading("💾 Nettoyage de Disque");
    ui.separator();

    // Options de nettoyage
    ui.label("📋 Options de nettoyage :");
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.disk_options.clean_temp_files, "🗃️ Fichiers temporaires");
        ui.checkbox(&mut app.disk_options.clean_browser_cache, "🌐 Cache navigateurs");
    });
    
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.disk_options.clean_thumbnails, "🖼️ Miniatures");
        ui.checkbox(&mut app.disk_options.clean_recycle_bin, "🗑️ Corbeille");
    });

    ui.horizontal(|ui| {
        ui.checkbox(&mut app.disk_options.clean_system_cache, "⚙️ Cache système");
    });

    ui.separator();

    // Optimisations spécifiques Windows
    ui.label("🪟 Optimisations Windows :");
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.disk_options.win11_optimizations, "Windows 11");
        ui.checkbox(&mut app.disk_options.win10_optimizations, "Windows 10");
    });

    ui.separator();

    // Boutons d'action
    let is_busy = app.disk_cleaning_promise.is_some();

    ui.horizontal(|ui| {
        if ui.add_enabled(!is_busy, egui::Button::new("🔍 Aperçu")).clicked() {
            // Lance l'aperçu en arrière-plan
            let options = app.disk_options.clone();
            app.disk_cleaning_promise = Some(Promise::spawn_thread("disk_scan", move || {
                match crate::disk::scan_disk_with_options(options) {
                    Ok(results) => results,
                    Err(_) => crate::disk::DiskCleaningResults::new(), // Résultat vide en cas d'erreur
                }
            }));
        }

        if ui.add_enabled(!is_busy, egui::Button::new("🧹 Nettoyer")).clicked() {
            // Lance le nettoyage en arrière-plan  
            let options = app.disk_options.clone();
            app.disk_cleaning_promise = Some(Promise::spawn_thread("disk_clean", move || {
                match tokio::runtime::Runtime::new().unwrap().block_on(async {
                    crate::disk::clean_disk_with_options(options).await
                }) {
                    Ok(results) => results,
                    Err(_) => crate::disk::DiskCleaningResults::new(), // Résultat vide en cas d'erreur
                }
            }));
        }
    });

    // Gestion des promises et barre de progression
    if let Some(promise) = &app.disk_cleaning_promise {
        if let Some(result) = promise.ready() {
            // Promise terminée, récupère le résultat directement
            app.last_disk_cleaned_results = Some(result.clone());
            app.disk_cleaning_promise = None; // Nettoie la promise
        } else {
            // En cours d'exécution
            ui.separator();
            ui.label("🔄 Opération en cours...");
            ui.add(ProgressBar::new(0.5).show_percentage());
        }
    }

    // Résultats
    if let Some(results) = &app.last_disk_cleaned_results {
        ui.separator();
        ui.label("✅ Derniers résultats :");
        ui.label(format!("📁 Fichiers temporaires: {}", results.temp_files_cleaned));
        ui.label(format!("💾 Espace libéré: {:.2} MB", results.total_space_freed as f64 / 1024.0 / 1024.0));
    }
} 