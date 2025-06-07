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

// Structure principale pour l'application
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
        // Préchargement des ressources au démarrage avec le minimum nécessaire
        let ctx = &cc.egui_ctx;
        
        // Charger uniquement le logo au démarrage pour accélérer le lancement
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
                // Fallback logo en cas d'erreur
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
        };        Self {
            cleaning_promise: None,
            last_results: None,
            disk_cleaning_promise: None,
            last_disk_results: None,
            disk_preview: None,
            disk_cleaning_options: DiskCleaningOptions::default(),            services_promise: None,
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
            ram_icon_texture: None, // Chargé à la demande
            last_update: std::time::Instant::now(),
            active_tab: ActiveTab::Memory,
            last_memory_refresh: Instant::now(),
        }
    }

    fn start_cleaning(&mut self) {
        if self.cleaning_promise.is_some() {
            return; // Ne pas démarrer un nouveau nettoyage si un est en cours
        }

        self.cleaning_progress = 0.0; // Réinitialiser la progression
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
    }    fn start_disk_cleaning(&mut self) {
        if self.disk_cleaning_promise.is_some() {
            return; // Ne pas démarrer un nouveau nettoyage si un est en cours
        }

        self.disk_cleaning_progress = 0.0;
        let options = self.disk_cleaning_options.clone();
        self.disk_cleaning_promise = Some(Promise::spawn_thread("disk_cleaning", move || {
            // Utiliser tokio runtime pour exécuter le code async
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
    }    fn start_services_optimization(&mut self) {
        if self.services_promise.is_some() {
            return; // Ne pas démarrer une nouvelle optimisation si une est en cours
        }

        self.services_progress = 0.0;
        
        // Clone selected services for the async operation
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
        const GB: f64 = 1_073_741_824.0; // 1024^3
        const MB: f64 = 1_048_576.0;     // 1024^2
        
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
        const GB: f64 = 1_073_741_824.0; // 1024^3
        const MB: f64 = 1_048_576.0;     // 1024^2
        
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

    // ...existing code...
}

impl eframe::App for CleanRamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {        // Configurer le thème avec des couleurs sobres et texte blanc par défaut
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = Color32::from_rgb(25, 25, 25);
        style.visuals.panel_fill = Color32::from_rgb(32, 32, 32);
        
        // Texte blanc par défaut pour tous les widgets
        style.visuals.widgets.inactive.fg_stroke.color = Color32::WHITE;
        style.visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
        style.visuals.widgets.active.fg_stroke.color = Color32::WHITE;
        style.visuals.widgets.open.fg_stroke.color = Color32::WHITE;
        style.visuals.widgets.noninteractive.fg_stroke.color = Color32::WHITE;
        
        ctx.set_style(style);

        // Rafraîchir automatiquement les informations mémoire toutes les 2 secondes
        self.refresh_memory_info();
        
        // Demander un repaint dans 2 secondes pour le rafraîchissement automatique
        ctx.request_repaint_after(Duration::from_secs(2));

        // Mise à jour du timestamp
        self.last_update = std::time::Instant::now();        // Vérifier si le nettoyage est terminé
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
        }        // Vérifier si le nettoyage de disque est terminé
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
        }        // Vérifier si l'optimisation des services est terminée
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

        // Interface unique avec CentralPanel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                // En-tête avec logo et titre côte à côte
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        if let Some(texture) = &self.logo_texture {
                            // Taille fixe pour le logo dans l'interface
                            let logo_size = Vec2::new(64.0, 64.0);
                            ui.add(egui::Image::new(texture).fit_to_exact_size(logo_size));
                            ui.add_space(10.0);
                        }
                        ui.heading("GameBooster");
                    });
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);                // Barre d'onglets responsive
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing.x = 2.0;
                    let available_width = ui.available_width() - 10.0; // Marge pour éviter le débordement
                    let tab_width = available_width / 5.0;
                    
                    // Style pour les onglets avec texte blanc par défaut
                    let mut tab_style = ui.style_mut().clone();
                    tab_style.visuals.widgets.inactive.fg_stroke.color = Color32::WHITE;
                    tab_style.visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
                    tab_style.visuals.widgets.active.fg_stroke.color = Color32::WHITE;
                    
                    ui.set_style(tab_style);
                    
                    if ui.add_sized([tab_width, 40.0], egui::SelectableLabel::new(
                        self.active_tab == ActiveTab::Memory, "🧠 Mémoire")).clicked() {
                        self.active_tab = ActiveTab::Memory;
                    }
                    if ui.add_sized([tab_width, 40.0], egui::SelectableLabel::new(
                        self.active_tab == ActiveTab::DiskCleaning, "💾 Disque")).clicked() {
                        self.active_tab = ActiveTab::DiskCleaning;
                    }
                    if ui.add_sized([tab_width, 40.0], egui::SelectableLabel::new(
                        self.active_tab == ActiveTab::Scheduler, "⏰ Scheduler")).clicked() {
                        self.active_tab = ActiveTab::Scheduler;
                    }
                    if ui.add_sized([tab_width, 40.0], egui::SelectableLabel::new(
                        self.active_tab == ActiveTab::Services, "⚙️ Services")).clicked() {
                        self.active_tab = ActiveTab::Services;
                    }
                    if ui.add_sized([tab_width, 40.0], egui::SelectableLabel::new(
                        self.active_tab == ActiveTab::NetworkLimiter, "🌐 Réseau")).clicked() {
                        self.active_tab = ActiveTab::NetworkLimiter;
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();                ui.add_space(15.0);                // Contenu selon l'onglet actif (responsive - prend tout l'espace disponible)
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), ui.available_height() - 20.0),
                    Layout::top_down(Align::Center),
                    |ui| {
                        // Le contenu s'étire pour prendre tout l'espace
                        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                            match self.active_tab {
                                ActiveTab::Memory => self.show_memory_tab(ui),
                                ActiveTab::DiskCleaning => self.show_disk_cleaning_tab(ui),
                                ActiveTab::Scheduler => self.show_scheduler_tab(ui),
                                ActiveTab::Services => self.show_services_tab(ui),
                                ActiveTab::NetworkLimiter => self.show_network_limiter_tab(ui),
                            }
                        });
                    },
                );
            });
        });        // Demander une mise à jour continue pendant les nettoyages
        if self.cleaning_promise.is_some() || self.disk_cleaning_promise.is_some() || self.services_promise.is_some() {
            ctx.request_repaint();
        }
    }
}

impl CleanRamApp {
    fn show_memory_tab(&mut self, ui: &mut egui::Ui) {
        // Informations mémoire avec formatage en Go/Mo
        let (total, avail) = self.system_memory_info;
        let used = total - avail;
        let usage_percent = (used as f64 / total as f64) * 100.0;
        
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("📊 État de la mémoire").size(16.0).strong());
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Mémoire totale:");
                    ui.label(RichText::new(Self::format_memory(total)).strong());
                });
                
                ui.horizontal(|ui| {
                    ui.label("Mémoire utilisée:");
                    ui.label(RichText::new(Self::format_memory(used)).color(Color32::from_rgb(255, 100, 100)));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Mémoire libre:");
                    ui.label(RichText::new(Self::format_memory(avail)).color(Color32::from_rgb(100, 255, 100)));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Utilisation:");
                    ui.label(RichText::new(format!("{:.1}%", usage_percent)).strong());
                });
                
                // Barre de progression pour l'utilisation mémoire
                ui.add_space(10.0);
                let progress = used as f32 / total as f32;
                ui.add(egui::ProgressBar::new(progress)
                    .desired_width(300.0)
                    .text(format!("{:.1}% utilisé", usage_percent)));
            });
        });
        
        ui.add_space(20.0);
          // Bouton de nettoyage RAM
        if self.cleaning_promise.is_none() {
            let button_text = "🧹 Nettoyer la mémoire cache";
            let button_size = Vec2::new(250.0, 40.0);
            
            let button = egui::Button::new(RichText::new(button_text).size(14.0).color(Color32::WHITE))
                .fill(Color32::from_rgb(0, 150, 255))
                .rounding(Rounding::same(8.0))
                .min_size(button_size);
            
            if ui.add(button).clicked() {
                self.start_cleaning();
            }
        } else {
            // Affichage du nettoyage en cours
            ui.label(RichText::new("🔄 Nettoyage en cours...").size(14.0).color(Color32::WHITE));
            ui.add(egui::ProgressBar::new(self.cleaning_progress)
                .desired_width(250.0)
                .text("Nettoyage..."));
        }
        
        ui.add_space(20.0);
          // Résultats du dernier nettoyage
        if let Some(ref results) = self.last_results {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("📈 Dernier nettoyage").size(16.0).strong());
                    ui.add_space(10.0);
                    
                    if results.has_error {
                        ui.label(RichText::new("❌ Erreur lors du nettoyage").color(Color32::RED));
                        ui.label(&results.error_message);
                    } else {
                        let total_freed = self.calculate_total_freed();
                        ui.label(RichText::new(format!("✅ Mémoire libérée: {}", Self::format_memory(total_freed)))
                            .color(Color32::from_rgb(100, 255, 100))
                            .size(14.0));
                          
                        ui.horizontal(|ui| {
                            ui.label("Processus nettoyés:");
                            ui.label(format!("{}", results.processes.len()));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Mémoire avant:");
                            ui.label(Self::format_memory(results.total_memory_before));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Mémoire après:");
                            ui.label(Self::format_memory(results.total_memory_after));
                        });

                        // Liste des processus avec RAM libérée, triée par ordre décroissant
                        if !results.processes.is_empty() {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(5.0);
                            ui.label(RichText::new("🔍 Processus nettoyés (triés par RAM libérée):").size(14.0).strong());
                            ui.add_space(5.0);
                            
                            // Créer une copie triée des processus
                            let mut sorted_processes = results.processes.clone();
                            sorted_processes.sort_by(|a, b| b.memory_freed.cmp(&a.memory_freed));
                            
                            // Afficher dans un scrollable si beaucoup de processus
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        for process in &sorted_processes {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("•").color(Color32::LIGHT_BLUE));
                                                ui.label(&process.name);
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
    }    fn show_disk_cleaning_tab(&mut self, ui: &mut egui::Ui) {
        // Charger l'aperçu si pas encore fait
        self.load_disk_preview();
        
        ui.label(RichText::new("💾 Nettoyage de Disque Avancé").size(18.0).strong());
        ui.add_space(15.0);
        
        // Options de nettoyage avec checkboxes
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("🛠️ Options de nettoyage").size(16.0).strong());
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.disk_cleaning_options.clean_temp_files, "");
                    ui.label("📁 Fichiers temporaires Windows");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("(Recommandé)").color(Color32::LIGHT_GREEN).size(12.0));
                    });
                });
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.disk_cleaning_options.clean_browser_cache, "");
                    ui.label("🌐 Cache des navigateurs");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("(Recommandé)").color(Color32::LIGHT_GREEN).size(12.0));
                    });
                });
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.disk_cleaning_options.clean_thumbnails, "");
                    ui.label("🖼️ Miniatures Windows");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("(Sûr)").color(Color32::LIGHT_BLUE).size(12.0));
                    });
                });
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.disk_cleaning_options.clean_recycle_bin, "");
                    ui.label("🗑️ Corbeille Windows");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("(Attention!)").color(Color32::YELLOW).size(12.0));
                    });
                });                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.disk_cleaning_options.clean_system_cache, "");
                        ui.label("⚙️ Cache système");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new("(Avancé)").color(Color32::from_rgb(255, 165, 0)).size(12.0));
                        });
                    });
            });
        });
        
        ui.add_space(15.0);        // Aperçu de l'espace récupérable avec calcul dynamique
        let mut should_refresh_preview = false;
        if let Some(ref preview) = self.disk_preview {
            let preview_clone = preview.clone();
            let options_clone = self.disk_cleaning_options.clone();
            
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("📊 Estimation de l'espace récupérable").size(16.0).strong());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let refresh_button = egui::Button::new(RichText::new("🔄").size(14.0))
                                .fill(Color32::from_rgb(70, 70, 70))
                                .rounding(Rounding::same(4.0));
                            if ui.add(refresh_button).on_hover_text("Actualiser l'estimation").clicked() {
                                should_refresh_preview = true;
                            }
                        });
                    });
                    ui.add_space(10.0);
                    
                    // Calcul dynamique basé sur les options sélectionnées
                    let mut total_estimated = 0u64;
                      if options_clone.clean_temp_files && preview_clone.temp_files_cleaned > 0 {
                        total_estimated += preview_clone.temp_files_cleaned;
                        ui.horizontal(|ui| {
                            ui.label("• Fichiers temporaires:");
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_size(preview_clone.temp_files_cleaned))
                                    .color(Color32::LIGHT_GREEN));
                            });
                        });
                    }
                    
                    if options_clone.clean_browser_cache && preview_clone.cache_cleaned > 0 {
                        total_estimated += preview_clone.cache_cleaned;
                        ui.horizontal(|ui| {
                            ui.label("• Cache navigateurs:");
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_size(preview_clone.cache_cleaned))
                                    .color(Color32::LIGHT_GREEN));
                            });
                        });
                    }
                    
                    if options_clone.clean_thumbnails && preview_clone.thumbnails_cleaned > 0 {
                        total_estimated += preview_clone.thumbnails_cleaned;
                        ui.horizontal(|ui| {
                            ui.label("• Miniatures Windows:");
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_size(preview_clone.thumbnails_cleaned))
                                    .color(Color32::LIGHT_GREEN));
                            });
                        });
                    }
                    
                    // Affichage du total estimé dynamique
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Total estimé:").strong());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if total_estimated > 0 {
                                ui.label(RichText::new(Self::format_size(total_estimated))
                                    .color(Color32::from_rgb(100, 255, 100))
                                    .strong()
                                    .size(16.0));                            } else {
                                ui.label(RichText::new("Aucune option sélectionnée")
                                    .color(Color32::GRAY)
                                    .italics());
                            }
                        });
                    });
                });
            });
            
            ui.add_space(15.0);        } else {
            // Bouton pour charger l'aperçu si pas encore fait
            ui.horizontal(|ui| {
                let scan_button = egui::Button::new(RichText::new("🔍 Scanner l'espace récupérable").size(14.0).color(Color32::WHITE))
                    .fill(Color32::from_rgb(70, 130, 180))
                    .rounding(Rounding::same(6.0))
                    .min_size(Vec2::new(220.0, 30.0));
                
                if ui.add(scan_button).clicked() {
                    should_refresh_preview = true;
                }
            });
            ui.add_space(15.0);
        }
        
        // Gérer le rafraîchissement en dehors des closures
        if should_refresh_preview {
            self.disk_preview = None; // Force reload
            self.load_disk_preview();
        }
        
        // Bouton de nettoyage
        if self.disk_cleaning_promise.is_none() {
            let button_text = "🧹 Lancer le nettoyage sélectionné";
            let button_size = Vec2::new(280.0, 40.0);
            
            let button = egui::Button::new(RichText::new(button_text).size(14.0).color(Color32::WHITE))
                .fill(Color32::from_rgb(255, 140, 0))
                .rounding(Rounding::same(8.0))
                .min_size(button_size);
            
            if ui.add(button).clicked() {
                self.start_disk_cleaning();
            }
        } else {
            // Affichage du nettoyage en cours
            ui.label(RichText::new("🔄 Nettoyage du disque en cours...").size(14.0).color(Color32::WHITE));
            ui.add(egui::ProgressBar::new(self.disk_cleaning_progress)
                .desired_width(280.0)
                .text("Nettoyage..."));
        }
        
        ui.add_space(20.0);        // Résultats du dernier nettoyage de disque avec logs détaillés
        let mut should_refresh_preview_results = false;
        if let Some(ref results) = self.last_disk_results {
            let results_clone = results.clone();
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("📈 Rapport du dernier nettoyage").size(16.0).strong());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let rescan_button = egui::Button::new(RichText::new("🔍 Re-scanner").size(12.0).color(Color32::WHITE))
                                .fill(Color32::from_rgb(70, 130, 180))
                                .rounding(Rounding::same(4.0))
                                .min_size(Vec2::new(100.0, 25.0));
                            if ui.add(rescan_button).on_hover_text("Scanner à nouveau l'espace récupérable").clicked() {
                                should_refresh_preview_results = true;
                            }
                        });
                    });
                    ui.add_space(10.0);
                    
                    if !results_clone.errors.is_empty() {
                        ui.label(RichText::new("⚠️ Erreurs rencontrées:").color(Color32::YELLOW));
                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for error in &results_clone.errors {
                                    ui.label(RichText::new(format!("• {}", error)).color(Color32::from_rgb(255, 200, 100)));
                                }
                            });
                        ui.add_space(10.0);
                    }
                    
                    ui.label(RichText::new(format!("✅ Espace total libéré: {}", Self::format_size(results_clone.total_space_freed)))
                        .color(Color32::from_rgb(100, 255, 100))
                        .size(14.0)
                        .strong());                    
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                    
                    ui.label(RichText::new("📋 Détail par catégorie:").size(14.0).strong());
                    ui.add_space(5.0);
                    
                    if results_clone.temp_files_cleaned > 0 {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("📁").size(16.0));
                            ui.label("Fichiers temporaires:");
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_size(results_clone.temp_files_cleaned))
                                    .color(Color32::from_rgb(100, 255, 100))
                                    .strong());
                            });
                        });
                    }
                    
                    if results_clone.cache_cleaned > 0 {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🌐").size(16.0));
                            ui.label("Cache navigateurs:");
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(Self::format_size(results_clone.cache_cleaned))
                                    .color(Color32::from_rgb(100, 255, 100))
                                    .strong());
                            });
                        });
                    }
                    
                    if results_clone.thumbnails_cleaned > 0 {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🖼️").size(16.0));
                            ui.label("Miniatures Windows:");
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
                        ui.label("📊 Fichiers traités:");
                        ui.label(RichText::new(format!("{}", results_clone.files_processed))
                            .color(Color32::LIGHT_BLUE)
                            .strong());
                    });
                    
                    if let Some(duration) = results_clone.duration {
                        ui.horizontal(|ui| {
                            ui.label("⏱️ Durée du nettoyage:");
                            ui.label(RichText::new(format!("{:.2}s", duration.as_secs_f64()))
                                .color(Color32::LIGHT_BLUE)
                                .strong());                        });
                    }
                });
            });
        }
        
        // Gérer le rafraîchissement en dehors des closures
        if should_refresh_preview_results {
            self.disk_preview = None; // Force reload
            self.load_disk_preview();
        }
    }
    
    fn show_scheduler_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("⏰ Planificateur de Tâches").size(18.0).strong());
            ui.add_space(20.0);
            
            ui.label("🚧 Fonctionnalité en développement");
            ui.add_space(10.0);
            
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Fonctionnalités prévues:");
                    ui.label("• Planification automatique du nettoyage RAM");
                    ui.label("• Planification du nettoyage disque");
                    ui.label("• Déclenchement au démarrage système");
                    ui.label("• Programmation périodique (horaire/quotidienne)");
                });
            });
        });
    }    fn show_services_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("[SERVICES] Optimisation des Services").size(18.0).strong().color(Color32::WHITE));
            ui.add_space(20.0);
            
            // Section Windows Defender
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("[DEFENDER] Windows Defender").size(16.0).strong().color(Color32::WHITE));
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Status:").color(Color32::WHITE));
                        let status_color = if self.defender_enabled { 
                            Color32::from_rgb(100, 255, 100) 
                        } else { 
                            Color32::from_rgb(255, 100, 100) 
                        };
                        let status_text = if self.defender_enabled { "[ON] Activé" } else { "[OFF] Désactivé" };
                        ui.label(RichText::new(status_text).color(status_color).strong());
                    });
                    
                    ui.add_space(5.0);
                    ui.label(RichText::new("! ATTENTION: Désactiver Windows Defender réduit la sécurité du système")
                        .color(Color32::YELLOW)
                        .size(12.0));
                    
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("[REFRESH] Vérifier Status").color(Color32::WHITE))
                            .clicked() {
                            self.update_defender_status();
                        }
                        
                        if self.defender_enabled {
                            if ui.button(RichText::new("[DISABLE] Désactiver temporairement").color(Color32::WHITE))
                                .clicked() {
                                self.defender_enabled = false; // Will be updated by actual check
                            }
                        } else {
                            if ui.button(RichText::new("[ENABLE] Réactiver").color(Color32::WHITE))
                                .clicked() {
                                self.defender_enabled = true; // Will be updated by actual check
                            }
                        }
                    });
                });
            });            
            ui.add_space(20.0);
            
            // Section Services Gaming
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("[GAMING] Services Gaming").size(16.0).strong().color(Color32::WHITE));
                    ui.add_space(10.0);
                    
                    ui.label(RichText::new("Services pouvant être optimisés pour les jeux:")
                        .color(Color32::WHITE));
                    ui.add_space(5.0);
                    
                    // Liste des services avec leurs statuts et checkboxes
                    let gaming_services = vec![
                        ("Windows Search", "WSearch", "Indexation des fichiers"),
                        ("Windows Update", "wuauserv", "Mises à jour automatiques"),
                        ("Superfetch", "SysMain", "Préchargement des applications"),
                        ("Print Spooler", "Spooler", "Service d'impression"),
                        ("Tablet PC Input Service", "TabletInputService", "Saisie tactile"),
                        ("Windows Error Reporting", "WerSvc", "Rapports d'erreurs"),
                    ];
                    
                    // Boutons de sélection globale
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("[SELECT ALL] Tout sélectionner").color(Color32::WHITE))
                            .clicked() {
                            for (_, service_name, _) in &gaming_services {
                                self.selected_services.insert(service_name.to_string(), true);
                            }
                        }
                        
                        if ui.button(RichText::new("[DESELECT ALL] Tout désélectionner").color(Color32::WHITE))
                            .clicked() {
                            for (_, service_name, _) in &gaming_services {
                                self.selected_services.insert(service_name.to_string(), false);
                            }
                        }
                    });
                    
                    ui.add_space(10.0);
                    
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (display_name, service_name, description) in gaming_services {
                                ui.horizontal(|ui| {
                                    // Checkbox pour sélection individuelle
                                    let mut selected = self.selected_services.get(service_name).unwrap_or(&false).clone();
                                    if ui.checkbox(&mut selected, "").clicked() {
                                        self.selected_services.insert(service_name.to_string(), selected);
                                    }
                                    
                                    // Status indicator
                                    let status = self.get_cached_service_status(service_name);
                                    let (status_icon, status_color) = match status.as_str() {
                                        "Running" => ("[RUN]", Color32::from_rgb(100, 255, 100)),
                                        "Stopped" => ("[STOP]", Color32::from_rgb(255, 100, 100)),
                                        "Starting" => ("[START]", Color32::YELLOW),
                                        "Stopping" => ("[STOP]", Color32::YELLOW),
                                        _ => ("[UNK]", Color32::GRAY),
                                    };
                                    
                                    ui.label(RichText::new(status_icon).size(12.0).color(status_color));
                                    ui.label(RichText::new(display_name).color(Color32::WHITE).strong());
                                    ui.label(RichText::new(format!("({})", status)).color(status_color).size(12.0));
                                });
                                ui.label(RichText::new(format!("  └─ {}", description))
                                    .color(Color32::GRAY)
                                    .size(11.0));
                                ui.add_space(5.0);
                            }
                        });                    
                    ui.add_space(10.0);
                    
                    // Boutons d'actions
                    if self.services_promise.is_none() {
                        ui.horizontal(|ui| {
                            let selected_count = self.selected_services.values().filter(|&&v| v).count();
                            
                            let optimize_button = egui::Button::new(RichText::new(format!("[OPTIMIZE] Optimiser {} Services", selected_count))
                                .size(14.0)
                                .color(Color32::WHITE))
                                .fill(Color32::from_rgb(255, 140, 0))
                                .rounding(Rounding::same(8.0))
                                .min_size(Vec2::new(200.0, 35.0));
                            
                            if ui.add(optimize_button).clicked() && selected_count > 0 {
                                self.start_services_optimization();
                            }
                            
                            if ui.button(RichText::new("[REFRESH] Actualiser Status").color(Color32::WHITE))
                                .clicked() {
                                self.refresh_services_status();
                            }
                        });
                    } else {
                        // Affichage de l'optimisation en cours
                        ui.label(RichText::new("[PROCESSING] Optimisation en cours...").size(14.0).color(Color32::WHITE));
                        ui.add(egui::ProgressBar::new(self.services_progress)
                            .desired_width(300.0)
                            .text("Optimisation des services..."));
                    }
                });
            });
              ui.add_space(20.0);
              // Résultats de la dernière optimisation
            let has_results = self.last_services_results.is_some();
            if has_results {
                let results_clone = self.last_services_results.as_ref().unwrap().clone();
                
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("[RESULTS] Résultats de l'optimisation").size(16.0).strong().color(Color32::WHITE));
                        ui.add_space(10.0);
                        
                        // Statistiques générales
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("[TIME] Durée:").color(Color32::WHITE));
                            if let Some(end_time) = results_clone.end_time {
                                let duration = end_time.signed_duration_since(results_clone.start_time);
                                ui.label(RichText::new(format!("{:.1}s", duration.num_milliseconds() as f64 / 1000.0))
                                    .color(Color32::from_rgb(100, 255, 100)));
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("[COUNT] Services optimisés:").color(Color32::WHITE));
                            ui.label(RichText::new(format!("{}", results_clone.services_optimized))
                                .color(Color32::from_rgb(100, 255, 100))
                                .strong());
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("[DEFENDER] Windows Defender:").color(Color32::WHITE));
                            let defender_status = if results_clone.defender_disabled { 
                                "Désactivé temporairement" 
                            } else { 
                                "Inchangé" 
                            };
                            let defender_color = if results_clone.defender_disabled { 
                                Color32::from_rgb(255, 200, 100) 
                            } else { 
                                Color32::from_rgb(100, 255, 100) 
                            };
                            ui.label(RichText::new(defender_status).color(defender_color));
                        });
                        
                        ui.add_space(10.0);
                        
                        // Détail des opérations
                        if !results_clone.operations.is_empty() {
                            ui.label(RichText::new("[DETAILS] Détail des opérations:").color(Color32::WHITE).strong());
                            ui.add_space(5.0);
                            
                            egui::ScrollArea::vertical()
                                .max_height(150.0)
                                .show(ui, |ui| {
                                    for operation in &results_clone.operations {
                                        ui.horizontal(|ui| {
                                            let (icon, color) = if operation.success {
                                                ("[OK]", Color32::from_rgb(100, 255, 100))
                                            } else {
                                                ("[ERR]", Color32::from_rgb(255, 100, 100))
                                            };
                                            
                                            ui.label(RichText::new(icon).size(12.0).color(color));
                                            ui.label(RichText::new(&operation.display_name).color(Color32::WHITE));
                                            ui.label(RichText::new(format!("({:?})", operation.action))
                                                .color(color)
                                                .size(11.0));
                                        });
                                        
                                        if !operation.success {
                                            if let Some(ref error) = operation.error_message {
                                                ui.label(RichText::new(format!("  └─ Erreur: {}", error))
                                                    .color(Color32::from_rgb(255, 200, 100))
                                                    .size(10.0));
                                            }
                                        }
                                        ui.add_space(3.0);
                                    }
                                });
                        }
                          // Erreurs générales
                        if !results_clone.errors.is_empty() {
                            ui.add_space(10.0);
                            ui.label(RichText::new("[WARNINGS] Erreurs rencontrées:").color(Color32::YELLOW));
                            egui::ScrollArea::vertical()
                                .max_height(100.0)
                                .show(ui, |ui| {
                                    for error in &results_clone.errors {
                                        ui.label(RichText::new(format!("• {}", error)).color(Color32::from_rgb(255, 200, 100)));
                                    }
                                });
                        }
                        
                        ui.add_space(10.0);
                        
                        // Boutons pour restaurer les services
                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("[RESTORE] Restaurer Services").color(Color32::WHITE))
                                .clicked() {
                                self.restore_selected_services();
                            }
                            
                            if ui.button(RichText::new("[CLEAR] Effacer Résultats").color(Color32::WHITE))
                                .clicked() {
                                self.last_services_results = None;
                            }
                        });
                    });
                });
            }
        });
    }
      fn show_network_limiter_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("[NETWORK] Limitation Réseau").size(18.0).strong().color(Color32::WHITE));
            ui.add_space(20.0);
            
            ui.label(RichText::new("[WIP] Fonctionnalité en développement").color(Color32::WHITE));
            ui.add_space(10.0);
              ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Fonctionnalités prévues:").color(Color32::WHITE));
                    ui.label(RichText::new("• Liste des processus et utilisation réseau").color(Color32::WHITE));
                    ui.label(RichText::new("• Limitation bande passante par processus").color(Color32::WHITE));
                    ui.label(RichText::new("• Blocage complet du réseau pour certains processus").color(Color32::WHITE));
                    ui.label(RichText::new("• Priorisation du trafic gaming").color(Color32::WHITE));
                });
            });
        });
    }
    
    // Méthodes pour la gestion des services    fn update_defender_status(&mut self) {
        // Lance la vérification du status Windows Defender en background
        tokio::spawn(async {
            match std::process::Command::new("powershell")
                .args(&["-Command", "Get-MpComputerStatus | Select-Object -ExpandProperty RealTimeProtectionEnabled"])
                .output() {
                Ok(output) => {
                    let status_str = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
                    // Le résultat sera utilisé lors du prochain refresh
                    println!("Defender status: {}", status_str);
                }
                Err(e) => {
                    eprintln!("Error checking Defender status: {}", e);
                }
            }
        });
    }
      fn get_cached_service_status(&mut self, service_name: &str) -> String {
        let now = Instant::now();
        
        // Vérifier si le status est en cache et encore valide (< 30 secondes)
        if let Some((status, last_check)) = self.services_status_cache.get(service_name) {
            if now.duration_since(*last_check) < Duration::from_secs(30) {
                return status.clone();
            }
        }
        
        // Récupérer le status actuel en background (optimisation PowerShell)
        let status = match get_service_status(service_name) {
            Ok(s) => s,
            Err(_) => {
                // Si l'appel échoue, lancer une vérification en background
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
        
        // Mettre à jour le cache
        self.services_status_cache.insert(service_name.to_string(), (status.clone(), now));
        
        status
    }
    
    fn refresh_services_status(&mut self) {
        // Vider le cache pour forcer une actualisation
        self.services_status_cache.clear();
    }
      fn restore_selected_services(&mut self) {
        // Restaurer seulement les services qui ont été optimisés
        if let Some(ref results) = self.last_services_results {
            let selected_services = self.selected_services.clone();
            
            // Lance la restauration en background
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