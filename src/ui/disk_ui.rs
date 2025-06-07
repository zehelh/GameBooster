use eframe::egui;
use egui::ProgressBar;
use crate::ui::app::CleanRamApp;

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
            // Simulation d'un aperçu
            ui.label("Fonctionnalité d'aperçu à implémenter...");
        }

        if ui.add_enabled(!is_busy, egui::Button::new("🧹 Nettoyer")).clicked() {
            // Simulation du nettoyage
            ui.label("Nettoyage lancé ! (simulation)");
        }
    });

    // Barre de progression
    if is_busy {
        ui.separator();
        ui.label("🔄 Nettoyage en cours...");
        ui.add(ProgressBar::new(0.5).show_percentage());
    }

    // Résultats
    if let Some(results) = &app.last_disk_cleaned_results {
        ui.separator();
        ui.label("✅ Derniers résultats :");
        ui.label(format!("📁 Fichiers temporaires: {}", results.temp_files_cleaned));
        ui.label(format!("💾 Espace libéré: {:.2} MB", results.total_space_freed as f64 / 1024.0 / 1024.0));
    }
} 