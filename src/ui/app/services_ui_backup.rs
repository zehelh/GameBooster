// Services UI implementation for CleanRamApp
use eframe::egui::{self, RichText, Color32, Vec2, Rounding};
use std::time::{Duration, Instant};
use crate::services::{get_service_status, optimize_selected_services_for_gaming};
use crate::services::gaming_services::restore_selected_services;
use super::CleanRamApp;

impl CleanRamApp {
    pub fn show_services_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Settings - Services Optimization").size(18.0).strong().color(Color32::WHITE));
            ui.add_space(20.0);
            
            // Section Windows Defender
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Shield - Windows Defender").size(16.0).strong().color(Color32::WHITE));
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Status:").color(Color32::WHITE));
                        let status_color = if self.defender_enabled { 
                            Color32::from_rgb(100, 255, 100) 
                        } else { 
                            Color32::from_rgb(255, 100, 100) 
                        };
                        let status_text = if self.defender_enabled { "[ON] Enabled" } else { "[OFF] Disabled" };
                        ui.label(RichText::new(status_text).color(status_color).strong());
                    });
                    
                    ui.add_space(5.0);
                    ui.label(RichText::new("Warning: Disabling Windows Defender reduces system security")
                        .color(Color32::YELLOW)
                        .size(12.0));
                    
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Refresh Status").color(Color32::WHITE))
                            .clicked() {
                            self.update_defender_status();
                        }
                        
                        if self.defender_enabled {
                            if ui.button(RichText::new("Pause Temporarily").color(Color32::WHITE))
                                .clicked() {
                                self.defender_enabled = false; // Will be updated by actual check
                            }
                        } else {
                            if ui.button(RichText::new("Play Reactivate").color(Color32::WHITE))
                                .clicked() {
                                self.defender_enabled = true; // Will be updated by actual check
                            }
                        }
                    });
                });
            });
            
            ui.add_space(20.0);
            
            // Section Services Gaming with checkboxes
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Game - Gaming Services").size(16.0).strong().color(Color32::WHITE));
                    ui.add_space(10.0);
                    
                    ui.label(RichText::new("Select services to optimize for gaming:")
                        .color(Color32::WHITE));
                    ui.add_space(5.0);
                    
                    // Liste des services avec checkboxes
                    let gaming_services = vec![
                        ("Windows Search", "WSearch", "File indexing service"),
                        ("Windows Update", "wuauserv", "Automatic updates"),
                        ("Superfetch", "SysMain", "Application preloading"),
                        ("Print Spooler", "Spooler", "Printing service"),
                        ("Tablet PC Input Service", "TabletInputService", "Touch input"),
                        ("Windows Error Reporting", "WerSvc", "Error reports"),
                    ];
                    
                    egui::ScrollArea::vertical()
                        .max_height(250.0)
                        .show(ui, |ui| {
                            for (display_name, service_name, description) in gaming_services {
                                ui.horizontal(|ui| {
                                    // Checkbox for service selection
                                    let mut selected = self.selected_services.get(service_name).unwrap_or(&false).clone();
                                    if ui.checkbox(&mut selected, "").changed() {
                                        self.selected_services.insert(service_name.to_string(), selected);
                                    }
                                    
                                    // Status indicator
                                    let status = self.get_cached_service_status(service_name);
                                    let (status_icon, status_color) = match status.as_str() {
                                        "Running" => ("[ON]", Color32::from_rgb(100, 255, 100)),
                                        "Stopped" => ("[OFF]", Color32::from_rgb(255, 100, 100)),
                                        "Starting" => ("[...] ", Color32::YELLOW),
                                        "Stopping" => ("[...] ", Color32::YELLOW),
                                        _ => ("[?]", Color32::GRAY),
                                    };
                                    
                                    ui.label(RichText::new(status_icon).size(12.0).color(status_color));
                                    ui.label(RichText::new(display_name).color(Color32::WHITE).strong());
                                    ui.label(RichText::new(format!("({})", status)).color(status_color).size(11.0));
                                });
                                ui.label(RichText::new(format!("  └─ {}", description))
                                    .color(Color32::GRAY)
                                    .size(11.0));
                                ui.add_space(5.0);
                            }
                        });
                    
                    ui.add_space(10.0);
                    
                    // Service selection summary
                    let selected_count = self.selected_services.values().filter(|&&v| v).count();
                    ui.label(RichText::new(format!("Selected services: {}/6", selected_count))
                        .color(Color32::YELLOW)
                        .size(12.0));
                    
                    ui.add_space(10.0);
                    
                    // Action buttons
                    if self.services_promise.is_none() {
                        ui.horizontal(|ui| {
                            let optimize_button = egui::Button::new(RichText::new("Rocket Optimize for Gaming")
                                .size(14.0)
                                .color(Color32::WHITE))
                                .fill(Color32::from_rgb(255, 140, 0))
                                .rounding(Rounding::same(8.0))
                                .min_size(Vec2::new(180.0, 35.0));
                            
                            if ui.add(optimize_button).clicked() && selected_count > 0 {
                                self.start_services_optimization();
                            }
                            
                            if ui.button(RichText::new("Refresh Refresh Status").color(Color32::WHITE))
                                .clicked() {
                                self.refresh_services_status();
                            }
                            
                            // Restore button (only show if there are previous results)
                            if self.last_services_results.is_some() {
                                if ui.button(RichText::new("Undo Restore Services").color(Color32::WHITE))
                                    .clicked() {
                                    self.restore_selected_services();
                                }
                            }
                        });
                        
                        // Selection helpers
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("Select All").color(Color32::WHITE).size(11.0))
                                .clicked() {
                                for (_, selected) in self.selected_services.iter_mut() {
                                    *selected = true;
                                }
                            }
                            
                            if ui.button(RichText::new("Select None").color(Color32::WHITE).size(11.0))
                                .clicked() {
                                for (_, selected) in self.selected_services.iter_mut() {
                                    *selected = false;
                                }
                            }
                            
                            if ui.button(RichText::new("Select Recommended").color(Color32::WHITE).size(11.0))
                                .clicked() {
                                // Reset all to false first
                                for (_, selected) in self.selected_services.iter_mut() {
                                    *selected = false;
                                }
                                // Then enable recommended ones
                                self.selected_services.insert("WSearch".to_string(), true);
                                self.selected_services.insert("wuauserv".to_string(), true);
                                self.selected_services.insert("SysMain".to_string(), true);
                                self.selected_services.insert("WerSvc".to_string(), true);
                            }
                        });
                    } else {
                        // Optimization in progress
                        ui.label(RichText::new("Refresh Optimization in progress...").size(14.0).color(Color32::WHITE));
                        ui.add(egui::ProgressBar::new(self.services_progress)
                            .desired_width(300.0)
                            .text("Optimizing services..."));
                    }
                });
            });
            
            ui.add_space(20.0);
            
            // Results from last optimization
            if let Some(ref results) = self.last_services_results {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Chart - Optimization Results").size(16.0).strong().color(Color32::WHITE));
                        ui.add_space(10.0);
                        
                        // General statistics
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Clock Duration:").color(Color32::WHITE));
                            if let Some(end_time) = results.end_time {
                                let duration = end_time.signed_duration_since(results.start_time);
                                ui.label(RichText::new(format!("{:.1}s", duration.num_milliseconds() as f64 / 1000.0))
                                    .color(Color32::from_rgb(100, 255, 100)));
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Wrench Services optimized:").color(Color32::WHITE));
                            ui.label(RichText::new(format!("{}", results.services_optimized))
                                .color(Color32::from_rgb(100, 255, 100))
                                .strong());
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Shield Windows Defender:").color(Color32::WHITE));
                            let defender_status = if results.defender_disabled { 
                                "Temporarily disabled" 
                            } else { 
                                "Unchanged" 
                            };
                            let defender_color = if results.defender_disabled { 
                                Color32::from_rgb(255, 200, 100) 
                            } else { 
                                Color32::from_rgb(100, 255, 100) 
                            };
                            ui.label(RichText::new(defender_status).color(defender_color));
                        });
                        
                        ui.add_space(10.0);
                        
                        // Operation details
                        if !results.operations.is_empty() {
                            ui.label(RichText::new("List Operation details:").color(Color32::WHITE).strong());
                            ui.add_space(5.0);
                            
                            egui::ScrollArea::vertical()
                                .max_height(150.0)
                                .show(ui, |ui| {
                                    for operation in &results.operations {
                                        ui.horizontal(|ui| {
                                            let (icon, color) = if operation.success {
                                                ("[OK]", Color32::from_rgb(100, 255, 100))
                                            } else {
                                                ("[ERR]", Color32::from_rgb(255, 100, 100))
                                            };
                                            
                                            ui.label(RichText::new(icon).size(11.0).color(color));
                                            ui.label(RichText::new(&operation.display_name).color(Color32::WHITE));
                                            ui.label(RichText::new(format!("({:?})", operation.action))
                                                .color(color)
                                                .size(11.0));
                                        });
                                        
                                        if !operation.success {
                                            if let Some(ref error) = operation.error_message {
                                                ui.label(RichText::new(format!("  └─ Error: {}", error))
                                                    .color(Color32::from_rgb(255, 200, 100))
                                                    .size(10.0));
                                            }
                                        }
                                        ui.add_space(3.0);
                                    }
                                });
                        }
                        
                        // General errors
                        if !results.errors.is_empty() {
                            ui.add_space(10.0);
                            ui.label(RichText::new("Warning Errors encountered:").color(Color32::YELLOW));
                            egui::ScrollArea::vertical()
                                .max_height(100.0)
                                .show(ui, |ui| {
                                    for error in &results.errors {
                                        ui.label(RichText::new(format!("• {}", error)).color(Color32::from_rgb(255, 200, 100)));
                                    }
                                });
                        }
                        
                        ui.add_space(10.0);
                        
                        // Clear results button
                        if ui.button(RichText::new("Trash Clear Results").color(Color32::WHITE))
                            .clicked() {
                            self.last_services_results = None;
                        }
                    });
                });
            }
        });
    }

    pub fn update_defender_status(&mut self) {
        // Launch PowerShell check in background
        tokio::spawn(async move {
            match tokio::process::Command::new("powershell")
                .args(&["-Command", "Get-MpComputerStatus | Select-Object -ExpandProperty RealTimeProtectionEnabled"])
                .output()
                .await {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let enabled = output_str.trim().eq_ignore_ascii_case("true");
                    println!("Windows Defender Real-time protection: {}", enabled);
                }
                Err(e) => {
                    eprintln!("Error checking Defender status: {}", e);
                }
            }
        });
    }

    pub fn get_cached_service_status(&mut self, service_name: &str) -> String {
        let now = Instant::now();
        
        // Check if status is cached and still valid (< 30 seconds)
        if let Some((status, last_check)) = self.services_status_cache.get(service_name) {
            if now.duration_since(*last_check) < Duration::from_secs(30) {
                return status.clone();
            }
        }
        
        // Get current status in background (PowerShell optimization)
        let status = match get_service_status(service_name) {
            Ok(s) => s,
            Err(_) => {
                // If call fails, launch background verification
                let service_name_clone = service_name.to_string();
                tokio::spawn(async move {
                    match std::process::Command::new("sc")
                        .args(&["query", &service_name_clone])
                        .output() {
                        Ok(output) => {
                            let output_str = String::from_utf8_lossy(&output.stdout);
                            if output_str.contains("RUNNING") {
                                println!("Service {} is running", service_name_clone);
                            } else if output_str.contains("STOPPED") {
                                println!("Service {} is stopped", service_name_clone);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error checking service {}: {}", service_name_clone, e);
                        }
                    }
                });
                "Unknown".to_string()
            }
        };
        
        // Update cache
        self.services_status_cache.insert(service_name.to_string(), (status.clone(), now));
        
        status
    }
    
    pub fn refresh_services_status(&mut self) {
        // Clear cache to force refresh
        self.services_status_cache.clear();
    }

    pub fn restore_selected_services(&mut self) {
        // Restore only services that were optimized
        if let Some(ref results) = self.last_services_results {
            let selected_services = self.selected_services.clone();
            
            // Launch restoration in background
            tokio::spawn(async move {
                match restore_selected_services(&selected_services).await {
                    Ok(operations) => {
                        println!("Restored {} services successfully", operations.len());
                        for op in operations {
                            if op.success {
                                println!("Restored: {}", op.display_name);
                            } else {
                                eprintln!("Failed to restore {}: {:?}", op.display_name, op.error_message);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error during services restoration: {}", e);
                    }
                }
            });
        }
    }
}
