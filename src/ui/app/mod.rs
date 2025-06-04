use anyhow::Result;
use chrono::Local;
use eframe::egui::{self, RichText, Align, Align2, Layout, TextStyle, Vec2, Color32, Rounding, Sense};
use poll_promise::Promise;
use image::load_from_memory;

use crate::memory::{CleaningResults, clean_memory, get_system_memory_info};
use crate::utils::format_size;

// Structure principale pour l'application
pub struct CleanRamApp {
    cleaning_promise: Option<Promise<Result<CleaningResults, String>>>,
    last_results: Option<CleaningResults>,
    show_admin_error: bool,
    cleaning_progress: f32,
    system_memory_info: (usize, usize),
    logo_texture: Option<egui::TextureHandle>,
    ram_icon_texture: Option<egui::TextureHandle>,
    last_update: std::time::Instant,
}

impl CleanRamApp {
    pub fn new(cc: &eframe::CreationContext<'_>, logo_bytes: &[u8], ram_icon_bytes: &[u8]) -> Self {
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
        };
          Self {
            cleaning_promise: None,
            last_results: None,
            show_admin_error: false,
            cleaning_progress: 0.0,
            system_memory_info: get_system_memory_info(),
            logo_texture,
            ram_icon_texture: None, // Chargé à la demande
            last_update: std::time::Instant::now(),
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
    }
}

impl eframe::App for CleanRamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Configurer le thème avec des couleurs sobres
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = Color32::from_rgb(25, 25, 25);
        style.visuals.panel_fill = Color32::from_rgb(32, 32, 32);
        ctx.set_style(style);

        // Mise à jour du timestamp
        self.last_update = std::time::Instant::now();

        // Vérifier si le nettoyage est terminé
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
                ui.add_space(10.0);
                
                // Interface RAM (unique)
                let (total, avail) = self.system_memory_info;
                    ui.horizontal(|ui| {
                        ui.label("Mémoire système:");
                        ui.label(format!("{} libres sur {} total", format_size(avail), format_size(total)));
                    });
                    ui.add_space(15.0);
                    
                    // Bouton de nettoyage RAM
                    if self.cleaning_promise.is_none() {
                        let button_text = "Nettoyer la mémoire cache";
                        let button_size = Vec2::new(250.0, 40.0);
                        let (rect, response) = ui.allocate_exact_size(button_size, Sense::click());
                        
                        let mut normal_color = Color32::from_rgb(30, 144, 255);  // Bleu normal
                        let hover_color = Color32::from_rgb(20, 100, 200);      // Bleu plus foncé au survol
                        
                        if response.hovered() {
                            normal_color = hover_color;
                        }
                        
                        ui.painter().rect_filled(
                            rect,
                            Rounding::same(5.0),
                            normal_color,
                        );
                        
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            button_text,
                            TextStyle::Button.resolve(ui.style()),
                            Color32::WHITE,
                        );
                        
                        if response.clicked() {
                            if !is_elevated::is_elevated() {
                                self.show_admin_error = true;
                            } else {
                                self.start_cleaning();
                            }
                        }
                    } else {
                        // Barre de progression du nettoyage RAM
                        ui.add_space(5.0);
                        let progress_bar = egui::widgets::ProgressBar::new(self.cleaning_progress)
                            .animate(true)
                            .show_percentage()
                            .desired_width(250.0);
                        ui.add(progress_bar);
                        
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(
                                RichText::new("Nettoyage en cours...")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(30, 144, 255))
                            );
                        });
                    }
                    
                    // Affichage des résultats du nettoyage RAM
                    if let Some(results) = &self.last_results {
                        ui.add_space(15.0);
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.heading("Résultats du nettoyage");
                            ui.horizontal(|ui| {
                                ui.label("Mémoire libérée:");
                                ui.label(
                                    RichText::new(format_size(results.total_freed()))
                                        .strong()
                                        .color(egui::Color32::from_rgb(0, 180, 0))
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Processus nettoyés:");
                                ui.label(RichText::new(format!("{}", results.processes.len())).strong());
                            });
                            ui.horizontal(|ui| {
                                let elapsed = if let Some(end_time) = results.end_time {
                                    (end_time - results.start_time).num_milliseconds() as f32 / 1000.0
                                } else {
                                    0.0
                                };
                                ui.label("Temps de nettoyage:");
                                ui.label(RichText::new(format!("{:.2}s", elapsed)).strong());
                            });
                            
                            // Détails des processus
                            ui.collapsing("Détails des processus", |ui| {
                                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                    let mut cleaned_processes = results.processes.clone();
                                    cleaned_processes.sort_by(|a, b| b.memory_freed.cmp(&a.memory_freed));
                                    
                                    for process in cleaned_processes {
                                        if process.memory_freed > 0 {
                                            ui.horizontal(|ui| {
                                                ui.label(&process.name);
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    ui.label(format_size(process.memory_freed));
                                                });
                                            });
                                        }
                                    }
                                });
                            });                        });
                    }
                }
                
                // Affichage du message d'erreur administrateur
                if self.show_admin_error {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("⚠️ Cette application nécessite des droits administrateur pour fonctionner correctement.")
                            .color(egui::Color32::from_rgb(255, 100, 100))
                    );
                    ui.label("Veuillez la redémarrer en tant qu'administrateur.");
                }
                
                // Version en bas
                ui.add_space(5.0);
                ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                    ui.add_space(5.0);
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .small()
                            .color(Color32::GRAY)
                    );
                });
            });
        });
        
        // Demander une mise à jour continue pendant le nettoyage
        if self.cleaning_promise.is_some() {
            ctx.request_repaint();
        }
    }
}