use eframe::egui;
use egui::ProgressBar;
use crate::ui::app::CleanRamApp;
use poll_promise::Promise;

pub fn draw_disk_tab(app: &mut CleanRamApp, ui: &mut egui::Ui) {
    ui.heading("💾 Disk Cleaning");
    ui.separator();

    // Cleaning options
    ui.label("📋 Cleaning Options:");
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.disk_options.clean_temp_files, "🗃️ Temporary files");
        ui.checkbox(&mut app.disk_options.clean_browser_cache, "🌐 Browser cache");
    });
    
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.disk_options.clean_thumbnails, "🖼️ Thumbnails");
        ui.checkbox(&mut app.disk_options.clean_recycle_bin, "🗑️ Recycle Bin");
    });

    ui.horizontal(|ui| {
        ui.checkbox(&mut app.disk_options.clean_system_cache, "⚙️ System cache");
    });

    ui.separator();    // Action buttons
    let is_busy = app.disk_scan_promise.is_some() || app.disk_clean_promise.is_some();

    ui.horizontal(|ui| {        if ui.add_enabled(!is_busy, egui::Button::new("🔍 Preview")).clicked() {
            let options = app.disk_options.clone();
            app.disk_scan_promise = Some(Promise::spawn_thread("disk_scan", move || {
                match crate::disk::scan_disk_with_options_sync(options) {
                    Ok(results) => results,
                    Err(_) => crate::disk::DiskScanResult::default(),
                }
            }));
        }        if ui.add_enabled(!is_busy, egui::Button::new("🧹 Clean")).clicked() {
            let options = app.disk_options.clone();
            app.disk_clean_promise = Some(Promise::spawn_thread("disk_clean", move || {
                match crate::disk::clean_disk_with_options_threaded(options) {
                    Ok(results) => results,
                    Err(_) => crate::disk::DiskCleanResult::default(),
                }
            }));
        }
    });

    // Handle scan promise
    if let Some(promise) = &app.disk_scan_promise {
        if let Some(result) = promise.ready() {
            app.last_disk_scan_result = Some(result.clone());
            app.disk_scan_promise = None;
        } else {
            ui.separator();
            ui.label("🔄 Scanning disk...");
            ui.add(ProgressBar::new(0.5).show_percentage());
        }
    }

    // Handle clean promise
    if let Some(promise) = &app.disk_clean_promise {
        if let Some(result) = promise.ready() {
            app.last_disk_clean_result = Some(result.clone());
            app.disk_clean_promise = None;
        } else {
            ui.separator();
            ui.label("🔄 Cleaning disk...");
            ui.add(ProgressBar::new(0.5).show_percentage());
        }
    }

    // Show cleaning results first if available, otherwise show scan results
    if let Some(clean_results) = &app.last_disk_clean_result {
        ui.separator();
        ui.label("✅ Cleaning completed:");
        
        let total_mb = clean_results.total_space_freed as f64 / 1024.0 / 1024.0;
        ui.label(format!("Total space freed: {:.2} MB", total_mb));

        egui::Grid::new("disk_clean_results_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Temporary files:");
                ui.label(format!("{:.2} MB", clean_results.temp_files_freed as f64 / 1024.0 / 1024.0));
                ui.end_row();

                ui.label("Browser cache:");
                ui.label(format!("{:.2} MB", clean_results.cache_freed as f64 / 1024.0 / 1024.0));
                ui.end_row();

                ui.label("Thumbnails:");
                ui.label(format!("{:.2} MB", clean_results.thumbnails_freed as f64 / 1024.0 / 1024.0));
                ui.end_row();

                ui.label("Recycle Bin:");
                ui.label(format!("{:.2} MB", clean_results.recycle_bin_freed as f64 / 1024.0 / 1024.0));
                ui.end_row();
            });
    } else if let Some(scan_results) = &app.last_disk_scan_result {
        ui.separator();
        ui.label("🔍 Scan results:");
        
        let total_mb = scan_results.total_space_to_free as f64 / 1024.0 / 1024.0;
        ui.label(format!("Total space to be freed: {:.2} MB", total_mb));

        egui::Grid::new("disk_scan_results_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Temporary files:");
                ui.label(format!("{:.2} MB", scan_results.temp_files_size as f64 / 1024.0 / 1024.0));
                ui.end_row();

                ui.label("Browser cache:");
                ui.label(format!("{:.2} MB", scan_results.cache_size as f64 / 1024.0 / 1024.0));
                ui.end_row();

                ui.label("Thumbnails:");
                ui.label(format!("{:.2} MB", scan_results.thumbnails_size as f64 / 1024.0 / 1024.0));
                ui.end_row();

                ui.label("Recycle Bin:");
                ui.label(format!("{:.2} MB", scan_results.recycle_bin_size as f64 / 1024.0 / 1024.0));
                ui.end_row();
            });
    }
}