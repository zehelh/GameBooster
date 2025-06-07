use anyhow::Result;
use chrono::Local;
use eframe::egui::{self, RichText, Align, Layout, Vec2, Color32, Rounding};
use poll_promise::Promise;
use image::load_from_memory;
use std::time::{Duration, Instant};

use crate::memory::{CleaningResults, clean_memory, get_system_memory_info};
use crate::disk::{DiskCleaningResults, DiskCleaningOptions, clean_disk_with_options, get_disk_cleaning_preview};
use crate::services::{ServicesOptimizationResults, optimize_services_for_gaming, optimize_selected_services_for_gaming, get_service_status};
use crate::services::gaming_services::restore_selected_services;

mod services_ui;

#[derive(Debug, Clone, PartialEq)]
enum ActiveTab {
    Memory,
    DiskCleaning,
    Services,
    NetworkLimiter,
    Scheduler,
}

// Main application structure
pub struct CleanRamApp {
    cleaning_promise: Option<Promise<Result<CleaningResults, String>>>,
    last_results: Option<CleaningResults>,
    disk_cleaning_promise: Option<Promise<Result<DiskCleaningResults, String>>>,
    last_disk_results: Option<DiskCleaningResults>,
    disk_preview: Option<DiskCleaningResults>,
    disk_cleaning_options: DiskCleaningOptions,
    services_promise: Option<Promise<Result<ServicesOptimizationResults, String>>>,
    last_services_results: Option<ServicesOptimizationResults>,
    services_status_cache: std::collections::HashMap<String, (String, Instant)>,
    selected_services: std::collections::HashMap<String, bool>, // Service selection state
    defender_enabled: bool,
    show_admin_error: bool,
    cleaning_progress: f32,
    disk_cleaning_progress: f32,
    services_progress: f32,
    system_memory_info: (usize, usize),
    logo_texture: Option<egui::TextureHandle>,
    ram_icon_texture: Option<egui::TextureHandle>,
    last_update: std::time::Instant,
    active_tab: ActiveTab,
    last_memory_refresh: Instant,
}

impl CleanRamApp {
    pub fn new(cc: &eframe::CreationContext<'_>, logo_bytes: &[u8], _ram_icon_bytes: &[u8]) -> Self {
        let ctx = &cc.egui_ctx;
        
        let logo_texture = match load_from_memory(logo_bytes) {
            Ok(image) => {
                let image = image.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
                let size = [image.width() as _, image.height() as _];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.into_raw();
                
                Some(ctx.load_texture(
                    "logo",
                    egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                    egui::TextureOptions::default(),
                ))
            },
            Err(_) => {
                let size = [256, 256];
                let mut pixels = vec![0; size[0] * size[1] * 4];
                for i in 0..pixels.len() / 4 {
                    pixels[i * 4 + 0] = 30;  // R
                    pixels[i * 4 + 1] = 144; // G
                    pixels[i * 4 + 2] = 255; // B
                    pixels[i * 4 + 3] = 255; // A
                }
                
                Some(ctx.load_texture(
                    "logo",
                    egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                    egui::TextureOptions::default(),
                ))
            }
        };
        
        Self {
            cleaning_promise: None,
            last_results: None,
            disk_cleaning_promise: None,
            last_disk_results: None,
            disk_preview: None,
            disk_cleaning_options: DiskCleaningOptions::default(),
            services_promise: None,
            last_services_results: None,
            services_status_cache: std::collections::HashMap::new(),
            selected_services: {
                let mut map = std::collections::HashMap::new();
                // Initialize all gaming services as selected by default
                map.insert("WSearch".to_string(), true);
                map.insert("wuauserv".to_string(), true);
                map.insert("SysMain".to_string(), true);
                map.insert("Spooler".to_string(), true);
                map.insert("TabletInputService".to_string(), false); // Less common
                map.insert("WerSvc".to_string(), true);
                map
            },
            defender_enabled: true,
            show_admin_error: false,
            cleaning_progress: 0.0,
            disk_cleaning_progress: 0.0,
            services_progress: 0.0,
            system_memory_info: get_system_memory_info(),
            logo_texture,
            ram_icon_texture: None,
            last_update: std::time::Instant::now(),
            active_tab: ActiveTab::Memory,
            last_memory_refresh: Instant::now(),
        }
    }

    fn start_cleaning(&mut self) {
        if self.cleaning_promise.is_some() {
            return;
        }

        self.cleaning_progress = 0.0;
        self.cleaning_promise = Some(Promise::spawn_thread("cleaning", || {
            match clean_memory() {
                Ok(results) => Ok(results),
                Err(e) => {
                    let mut results = CleaningResults::new();
                    results.has_error = true;
                    results.error_message = e.to_string();
                    results.is_completed = true;
                    results.end_time = Some(Local::now());
                    Ok(results)
                }
            }
        }));
    }

    fn start_disk_cleaning(&mut self) {
        if self.disk_cleaning_promise.is_some() {
            return;
        }

        self.disk_cleaning_progress = 0.0;
        let options = self.disk_cleaning_options.clone();
        self.disk_cleaning_promise = Some(Promise::spawn_thread("disk_cleaning", move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match clean_disk_with_options(options).await {
                    Ok(results) => Ok(results),
                    Err(e) => {
                        let mut results = DiskCleaningResults::new();
                        results.errors.push(e.to_string());
                        results.complete();
                        Ok(results)
                    }
                }
            })
        }));
    }

    fn start_services_optimization(&mut self) {
        if self.services_promise.is_some() {
            return;
        }

        self.services_progress = 0.0;
        let selected_services = self.selected_services.clone();
        
        self.services_promise = Some(Promise::spawn_thread("services_optimization", move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match optimize_selected_services_for_gaming(&selected_services).await {
                    Ok(results) => Ok(results),
                    Err(e) => {
                        let mut results = ServicesOptimizationResults::new();
                        results.errors.push(e.to_string());
                        results.complete();
                        Ok(results)
                    }
                }
            })
        }));
    }

    fn load_disk_preview(&mut self) {
        if self.disk_preview.is_none() {
            self.disk_preview = get_disk_cleaning_preview().ok();
        }
    }

    fn format_memory(bytes: usize) -> String {
        const GB: f64 = 1_073_741_824.0;
        const MB: f64 = 1_048_576.0;
        
        let bytes_f = bytes as f64;
        
        if bytes_f >= GB {
            format!("{:.2} Go", bytes_f / GB)
        } else if bytes_f >= MB {
            format!("{:.0} Mo", bytes_f / MB)
        } else {
            format!("{:.0} Ko", bytes_f / 1024.0)
        }
    }

    fn format_size(bytes: u64) -> String {
        const GB: f64 = 1_073_741_824.0;
        const MB: f64 = 1_048_576.0;
        
        let bytes_f = bytes as f64;
        
        if bytes_f >= GB {
            format!("{:.2} Go", bytes_f / GB)
        } else if bytes_f >= MB {
            format!("{:.0} Mo", bytes_f / MB)
        } else {
            format!("{:.0} Ko", bytes_f / 1024.0)
        }
    }

    fn refresh_memory_info(&mut self) {
        if self.last_memory_refresh.elapsed() >= Duration::from_secs(2) {
            self.system_memory_info = get_system_memory_info();
            self.last_memory_refresh = Instant::now();
        }
    }

    fn calculate_total_freed(&self) -> usize {
        if let Some(ref results) = self.last_results {
            results.total_freed()
        } else {
            0
        }
    }
}

impl eframe::App for CleanRamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Configure theme with white text by default
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = Color32::from_rgb(25, 25, 25);
        style.visuals.panel_fill = Color32::from_rgb(32, 32, 32);
        
        style.visuals.widgets.inactive.fg_stroke.color = Color32::WHITE;
        style.visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
        style.visuals.widgets.active.fg_stroke.color = Color32::WHITE;
        style.visuals.widgets.open.fg_stroke.color = Color32::WHITE;
        style.visuals.widgets.noninteractive.fg_stroke.color = Color32::WHITE;
        
        ctx.set_style(style);

        self.refresh_memory_info();
        ctx.request_repaint_after(Duration::from_secs(2));
        self.last_update = std::time::Instant::now();

        // Check promises status
        if let Some(promise) = &self.cleaning_promise {
            if let Some(result) = promise.ready() {
                if let Ok(results) = result {
                    self.last_results = Some(results.clone());
                    self.cleaning_progress = 1.0;
                }
                self.cleaning_promise = None;
            } else {
                if self.cleaning_progress < 0.95 {
                    self.cleaning_progress += 0.01;
                }
            }
        }

        if let Some(promise) = &self.disk_cleaning_promise {
            if let Some(result) = promise.ready() {
                if let Ok(results) = result {
                    self.last_disk_results = Some(results.clone());
                    self.disk_cleaning_progress = 1.0;
                }
                self.disk_cleaning_promise = None;
            } else {
                if self.disk_cleaning_progress < 0.95 {
                    self.disk_cleaning_progress += 0.01;
                }
            }
        }

        if let Some(promise) = &self.services_promise {
            if let Some(result) = promise.ready() {
                if let Ok(results) = result {
                    self.last_services_results = Some(results.clone());
                    self.services_progress = 1.0;
                }
                self.services_promise = None;
            } else {
                if self.services_progress < 0.95 {
                    self.services_progress += 0.01;
                }
            }
        }

        // Main interface
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                
                // Header with logo and title
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        if let Some(texture) = &self.logo_texture {
                            let logo_size = Vec2::new(64.0, 64.0);
                            ui.add(egui::Image::new(texture).fit_to_exact_size(logo_size));
                            ui.add_space(10.0);
                        }
                        ui.heading("GameBooster");
                    });
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // Responsive tab bar
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing.x = 2.0;
                    let available_width = ui.available_width() - 10.0;
                    let tab_width = available_width / 5.0;
                    
                    let mut tab_style = ui.style_mut().clone();
                    tab_style.visuals.widgets.inactive.fg_stroke.color = Color32::WHITE;
                    tab_style.visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
                    tab_style.visuals.widgets.active.fg_stroke.color = Color32::WHITE;
                    ui.set_style(tab_style);
                    
                    if ui.add_sized([tab_width, 40.0], egui::SelectableLabel::new(
                        self.active_tab == ActiveTab::Memory, "RAM Memory")).clicked() {
                        self.active_tab = ActiveTab::Memory;
                    }
                    if ui.add_sized([tab_width, 40.0], egui::SelectableLabel::new(
                        self.active_tab == ActiveTab::DiskCleaning, "HDD Disk")).clicked() {
                        self.active_tab = ActiveTab::DiskCleaning;
                    }
                    if ui.add_sized([tab_width, 40.0], egui::SelectableLabel::new(
                        self.active_tab == ActiveTab::Services, "COG Services")).clicked() {
                        self.active_tab = ActiveTab::Services;
                    }
                    if ui.add_sized([tab_width, 40.0], egui::SelectableLabel::new(
                        self.active_tab == ActiveTab::NetworkLimiter, "NET Network")).clicked() {
                        self.active_tab = ActiveTab::NetworkLimiter;
                    }
                    if ui.add_sized([tab_width, 40.0], egui::SelectableLabel::new(
                        self.active_tab == ActiveTab::Scheduler, "TIME Scheduler")).clicked() {
                        self.active_tab = ActiveTab::Scheduler;
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(15.0);

                // Content based on active tab (responsive)
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), ui.available_height() - 20.0),
                    Layout::top_down(Align::Center),
                    |ui| {
                        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                            match self.active_tab {
                                ActiveTab::Memory => self.show_memory_tab(ui),
                                ActiveTab::DiskCleaning => self.show_disk_cleaning_tab(ui),
                                ActiveTab::Services => self.show_services_tab(ui),
                                ActiveTab::NetworkLimiter => self.show_network_limiter_tab(ui),
                                ActiveTab::Scheduler => self.show_scheduler_tab(ui),
                            }
                        });
                    },
                );
            });
        });

        if self.cleaning_promise.is_some() || self.disk_cleaning_promise.is_some() || self.services_promise.is_some() {
            ctx.request_repaint();
        }
    }
}

impl CleanRamApp {
    fn show_memory_tab(&mut self, ui: &mut egui::Ui) {
        let (total, avail) = self.system_memory_info;
        let used = total - avail;
        let usage_percent = (used as f64 / total as f64) * 100.0;
        
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Chart Memory Status").size(16.0).strong().color(Color32::WHITE));
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Total memory:").color(Color32::WHITE));
                    ui.label(RichText::new(Self::format_memory(total)).strong().color(Color32::WHITE));
                });
                
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Used memory:").color(Color32::WHITE));
                    ui.label(RichText::new(Self::format_memory(used)).color(Color32::from_rgb(255, 100, 100)));
                });
                
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Free memory:").color(Color32::WHITE));
                    ui.label(RichText::new(Self::format_memory(avail)).color(Color32::from_rgb(100, 255, 100)));
                });
                
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Usage:").color(Color32::WHITE));
                    ui.label(RichText::new(format!("{:.1}%", usage_percent)).strong().color(Color32::WHITE));
                });
                
                ui.add_space(10.0);
                let progress = used as f32 / total as f32;
                ui.add(egui::ProgressBar::new(progress)
                    .desired_width(300.0)
                    .text(format!("{:.1}% used", usage_percent)));
            });
        });
        
        ui.add_space(20.0);

        // RAM cleaning button
        if self.cleaning_promise.is_none() {
            let button_text = "Brush Clean memory cache";
            let button_size = Vec2::new(250.0, 40.0);
            
            let button = egui::Button::new(RichText::new(button_text).size(14.0).color(Color32::WHITE))
                .fill(Color32::from_rgb(0, 150, 255))
                .rounding(Rounding::same(8.0))
                .min_size(button_size);
            
            if ui.add(button).clicked() {
                self.start_cleaning();
            }
        } else {
            ui.label(RichText::new("Refresh Cleaning in progress...").size(14.0).color(Color32::WHITE));
            ui.add(egui::ProgressBar::new(self.cleaning_progress)
                .desired_width(250.0)
                .text("Cleaning..."));
        }
        
        ui.add_space(20.0);

        // Results from last cleaning
        if let Some(ref results) = self.last_results {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Chart Last cleaning").size(16.0).strong().color(Color32::WHITE));
                    ui.add_space(10.0);
                    
                    if results.has_error {
                        ui.label(RichText::new("X Error during cleaning").color(Color32::RED));
                        ui.label(RichText::new(&results.error_message).color(Color32::WHITE));
                    } else {
                        let total_freed = self.calculate_total_freed();
                        ui.label(RichText::new(format!("OK Memory freed: {}", Self::format_memory(total_freed)))
                            .color(Color32::from_rgb(100, 255, 100))
                            .size(14.0));
                          
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Processes cleaned:").color(Color32::WHITE));
                            ui.label(RichText::new(format!("{}", results.processes.len())).color(Color32::WHITE));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Memory before:").color(Color32::WHITE));
                            ui.label(RichText::new(Self::format_memory(results.total_memory_before)).color(Color32::WHITE));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Memory after:").color(Color32::WHITE));
                            ui.label(RichText::new(Self::format_memory(results.total_memory_after)).color(Color32::WHITE));
                        });

                        // Process list sorted by freed RAM
                        if !results.processes.is_empty() {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(5.0);
                            ui.label(RichText::new("Magnify Cleaned processes (sorted by freed RAM):").size(14.0).strong().color(Color32::WHITE));
                            ui.add_space(5.0);
                            
                            let mut sorted_processes = results.processes.clone();
                            sorted_processes.sort_by(|a, b| b.memory_freed.cmp(&a.memory_freed));
                            
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        for process in &sorted_processes {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("•").color(Color32::LIGHT_BLUE));
                                                ui.label(RichText::new(&process.name).color(Color32::WHITE));
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    ui.label(RichText::new(Self::format_memory(process.memory_freed))
                                                        .color(Color32::from_rgb(100, 255, 100)));
                                                });
                                            });
                                        }
                                    });
                                });
                        }
                    }
                });
            });
        }
    }

    fn show_disk_cleaning_tab(&mut self, ui: &mut egui::Ui) {
        self.load_disk_preview();
        
        ui.label(RichText::new("HDD Advanced Disk Cleaning").size(18.0).strong().color(Color32::WHITE));
        ui.add_space(15.0);
        
        // Cleaning options with checkboxes
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Wrench Cleaning Options").size(16.0).strong().color(Color32::WHITE));
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.disk_cleaning_options.clean_temp_files, "");
                    ui.label(RichText::new("Folder Windows temporary files").color(Color32::WHITE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("(Recommended)").color(Color32::LIGHT_GREEN).size(12.0));
                    });
                });
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.disk_cleaning_options.clean_browser_cache, "");
                    ui.label(RichText::new("Globe Browser cache").color(Color32::WHITE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("(Recommended)").color(Color32::LIGHT_GREEN).size(12.0));
                    });
                });
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.disk_cleaning_options.clean_thumbnails, "");
                    ui.label(RichText::new("Image Windows thumbnails").color(Color32::WHITE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("(Safe)").color(Color32::LIGHT_BLUE).size(12.0));
                    });
                });
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.disk_cleaning_options.clean_recycle_bin, "");
                    ui.label(RichText::new("Trash Windows recycle bin").color(Color32::WHITE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("(Warning!)").color(Color32::YELLOW).size(12.0));
                    });
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.disk_cleaning_options.clean_system_cache, "");
                    ui.label(RichText::new("Settings System cache").color(Color32::WHITE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("(Advanced)").color(Color32::from_rgb(255, 165, 0)).size(12.0));
                    });
                });
            });
        });
        
        ui.add_space(15.0);

        // Preview recoverable space with dynamic calculation
        let mut should_refresh_preview = false;
        if let Some(ref preview) = self.disk_preview {
            let preview_clone = preview.clone();
            let options_clone = self.disk_cleaning_options.clone();
            
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Chart Estimation of recoverable space").size(16.0).strong().color(Color32::WHITE));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let refresh_button = egui::Button::new(RichText::new("Refresh").size(14.0).color(Color32::WHITE))
                                .fill(Color32::from_rgb(70, 70, 70))
                                .rounding(Rounding::same(4.0));
                            if ui.add(refresh_button).on_hover_text("Refresh estimation").clicked() {
                                should_refresh_preview = true;
                            }
                        });
                    });
                    ui.add_space(10.0);
                    
                    // Dynamic calculation based on selected options
                    let mut total_estimated = 0u64;

                    if options_clone.clean_temp_files && preview_clone.temp_files_cleaned > 0 {
                        total_estimated += preview_clone.temp_files_cleaned;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("• Temporary files:").color(Color32::WHITE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_size(preview_clone.temp_files_cleaned))
                                    .color(Color32::LIGHT_GREEN));
                            });
                        });
                    }
                    
                    if options_clone.clean_browser_cache && preview_clone.cache_cleaned > 0 {
                        total_estimated += preview_clone.cache_cleaned;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("• Browser cache:").color(Color32::WHITE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_size(preview_clone.cache_cleaned))
                                    .color(Color32::LIGHT_GREEN));
                            });
                        });
                    }
                    
                    if options_clone.clean_thumbnails && preview_clone.thumbnails_cleaned > 0 {
                        total_estimated += preview_clone.thumbnails_cleaned;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("• Windows thumbnails:").color(Color32::WHITE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_size(preview_clone.thumbnails_cleaned))
                                    .color(Color32::LIGHT_GREEN));
                            });
                        });
                    }
                    
                    // Display dynamic estimated total
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Estimated total:").strong().color(Color32::WHITE));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if total_estimated > 0 {
                                ui.label(RichText::new(Self::format_size(total_estimated))
                                    .color(Color32::from_rgb(100, 255, 100))
                                    .strong()
                                    .size(16.0));
                            } else {
                                ui.label(RichText::new("No option selected")
                                    .color(Color32::GRAY)
                                    .italics());
                            }
                        });
                    });
                });
            });
            
            ui.add_space(15.0);
        } else {
            ui.horizontal(|ui| {
                let scan_button = egui::Button::new(RichText::new("Magnify Scan recoverable space").size(14.0).color(Color32::WHITE))
                    .fill(Color32::from_rgb(70, 130, 180))
                    .rounding(Rounding::same(6.0))
                    .min_size(Vec2::new(220.0, 30.0));
                
                if ui.add(scan_button).clicked() {
                    should_refresh_preview = true;
                }
            });
            ui.add_space(15.0);
        }
        
        if should_refresh_preview {
            self.disk_preview = None;
            self.load_disk_preview();
        }
        
        // Cleaning button
        if self.disk_cleaning_promise.is_none() {
            let button_text = "Brush Start selected cleaning";
            let button_size = Vec2::new(280.0, 40.0);
            
            let button = egui::Button::new(RichText::new(button_text).size(14.0).color(Color32::WHITE))
                .fill(Color32::from_rgb(255, 140, 0))
                .rounding(Rounding::same(8.0))
                .min_size(button_size);
            
            if ui.add(button).clicked() {
                self.start_disk_cleaning();
            }
        } else {
            ui.label(RichText::new("Refresh Disk cleaning in progress...").size(14.0).color(Color32::WHITE));
            ui.add(egui::ProgressBar::new(self.disk_cleaning_progress)
                .desired_width(280.0)
                .text("Cleaning..."));
        }
        
        ui.add_space(20.0);

        // Results from last disk cleaning with detailed logs
        let mut should_refresh_preview_results = false;
        if let Some(ref results) = self.last_disk_results {
            let results_clone = results.clone();
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Chart Report from last cleaning").size(16.0).strong().color(Color32::WHITE));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let rescan_button = egui::Button::new(RichText::new("Magnify Re-scan").size(12.0).color(Color32::WHITE))
                                .fill(Color32::from_rgb(70, 130, 180))
                                .rounding(Rounding::same(4.0))
                                .min_size(Vec2::new(100.0, 25.0));
                            if ui.add(rescan_button).on_hover_text("Scan again for recoverable space").clicked() {
                                should_refresh_preview_results = true;
                            }
                        });
                    });
                    ui.add_space(10.0);
                    
                    if !results_clone.errors.is_empty() {
                        ui.label(RichText::new("Warning Errors encountered:").color(Color32::YELLOW));
                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for error in &results_clone.errors {
                                    ui.label(RichText::new(format!("• {}", error)).color(Color32::from_rgb(255, 200, 100)));
                                }
                            });
                        ui.add_space(10.0);
                    }
                    
                    ui.label(RichText::new(format!("OK Total space freed: {}", Self::format_size(results_clone.total_space_freed)))
                        .color(Color32::from_rgb(100, 255, 100))
                        .size(14.0)
                        .strong());
                    
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                    
                    ui.label(RichText::new("List Detail by category:").size(14.0).strong().color(Color32::WHITE));
                    ui.add_space(5.0);
                    
                    if results_clone.temp_files_cleaned > 0 {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Folder").size(16.0));
                            ui.label(RichText::new("Temporary files:").color(Color32::WHITE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_size(results_clone.temp_files_cleaned))
                                    .color(Color32::from_rgb(100, 255, 100))
                                    .strong());
                            });
                        });
                    }
                    
                    if results_clone.cache_cleaned > 0 {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Globe").size(16.0));
                            ui.label(RichText::new("Browser cache:").color(Color32::WHITE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_size(results_clone.cache_cleaned))
                                    .color(Color32::from_rgb(100, 255, 100))
                                    .strong());
                            });
                        });
                    }
                    
                    if results_clone.thumbnails_cleaned > 0 {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Image").size(16.0));
                            ui.label(RichText::new("Windows thumbnails:").color(Color32::WHITE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_size(results_clone.thumbnails_cleaned))
                                    .color(Color32::from_rgb(100, 255, 100))
                                    .strong());
                            });
                        });
                    }
                    
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Chart Files processed:").color(Color32::WHITE));
                        ui.label(RichText::new(format!("{}", results_clone.files_processed))
                            .color(Color32::LIGHT_BLUE)
                            .strong());
                    });
                    
                    if let Some(duration) = results_clone.duration {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Clock Cleaning duration:").color(Color32::WHITE));
                            ui.label(RichText::new(format!("{:.2}s", duration.as_secs_f64()))
                                .color(Color32::LIGHT_BLUE)
                                .strong());
                        });
                    }
                });
            });
        }
        
        if should_refresh_preview_results {
            self.disk_preview = None;
            self.load_disk_preview();
        }
    }

    fn show_scheduler_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Clock Task Scheduler").size(18.0).strong().color(Color32::WHITE));
            ui.add_space(20.0);
            
            ui.label(RichText::new("Construction Feature in development").color(Color32::WHITE));
            ui.add_space(10.0);
            
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Planned features:").color(Color32::WHITE));
                    ui.label(RichText::new("• Automatic memory cleaning scheduling").color(Color32::WHITE));
                    ui.label(RichText::new("• Disk cleaning on schedule").color(Color32::WHITE));
                    ui.label(RichText::new("• Gaming mode activation triggers").color(Color32::WHITE));
                    ui.label(RichText::new("• System optimization profiles").color(Color32::WHITE));
                });
            });
        });
    }

    fn show_network_limiter_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Globe Network Limiter").size(18.0).strong().color(Color32::WHITE));
            ui.add_space(20.0);
            
            ui.label(RichText::new("Construction Feature in development").color(Color32::WHITE));
            ui.add_space(10.0);
            
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Planned features:").color(Color32::WHITE));
                    ui.label(RichText::new("• Process network usage monitoring").color(Color32::WHITE));
                    ui.label(RichText::new("• Bandwidth limiting per process").color(Color32::WHITE));
                    ui.label(RichText::new("• Complete network blocking for processes").color(Color32::WHITE));
                    ui.label(RichText::new("• Gaming traffic prioritization").color(Color32::WHITE));
                });
            });
        });
    }
}
