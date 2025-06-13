use std::collections::HashSet;

use crate::disk::{DiskCleaningOptions, DiskCleaningResults};
use crate::memory::CleaningResults;
use crate::services::defender::DefenderStatus;
use crate::network::NetworkLimiter;

use eframe::egui;
// use image::load_from_memory; // Temporairement désactivé pour éviter les crashes
use poll_promise::Promise;

use crate::ui::{
    disk_ui, memory_ui, network_ui, services_ui, settings_ui, scheduler_ui
};

use crate::theme;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Tab {
    Memory,
    Optimization, // Renamed from Hdd
    Services,
    Scheduler,
    Network,
    Settings,
}

pub struct CleanRamApp {
    pub active_tab: Tab,
    pub theme: theme::Theme,
    pub ram_usage: f32,
    pub cleaning_promise: Option<Promise<CleaningResults>>,
    pub last_cleaned_results: Option<CleaningResults>,
    pub disk_options: DiskCleaningOptions,
    pub disk_cleaning_promise: Option<Promise<DiskCleaningResults>>,
    pub last_disk_cleaned_results: Option<DiskCleaningResults>,
    pub processes: HashSet<u32>,
    pub defender_status_promise: Option<Promise<Result<DefenderStatus, anyhow::Error>>>,
    pub defender_action_promise: Option<Promise<Result<bool, anyhow::Error>>>,
    pub last_defender_status: Option<Result<DefenderStatus, anyhow::Error>>,
    pub windows_version_string: String,
    pub logo: egui::TextureId,
    pub ram_icon: egui::TextureId,
    pub is_first_frame: bool,
    pub network_limiter: Option<NetworkLimiter>,
    pub process_search_text: String,
    pub speed_limit_input: String,
}

impl CleanRamApp {
    pub fn is_not_busy(&self) -> bool {
        // Only block UI during heavy operations, not status checks
        self.cleaning_promise.is_none() 
            && self.disk_cleaning_promise.is_none() 
            && self.defender_action_promise.is_none()
    }

    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Créer des textures simples sans charger d'images pour éviter les crashes
        let dummy_texture_id = egui::TextureId::default();
        
        let network_limiter = match crate::network::NetworkLimiter::new() {
            Ok(limiter) => {
                tracing::info!("✅ Network manager QoS initialized");
                Some(limiter)
            }
            Err(e) => {
                tracing::error!("❌ Failed to initialize network manager: {}", e);
                None
            }
        };

        let detected_os_version = crate::os_info::get_os_platform(); // Modifié pour obtenir le type d'OS
        tracing::info!("Detected OS Platform on startup (tracing): {}", detected_os_version);
        println!("Detected OS Platform on startup (println): {}", detected_os_version);

        Self {
            active_tab: Tab::Memory,
            theme: theme::dark_theme(),
            ram_usage: 0.0,
            cleaning_promise: None,
            last_cleaned_results: None,
            disk_options: DiskCleaningOptions::default(),
            disk_cleaning_promise: None,
            last_disk_cleaned_results: None,
            processes: HashSet::new(),
            defender_status_promise: None,
            defender_action_promise: None,
            last_defender_status: None,
            windows_version_string: detected_os_version, // Stocke la plateforme détectée
            logo: dummy_texture_id,
            ram_icon: dummy_texture_id,
            is_first_frame: true,
            network_limiter,
            process_search_text: String::new(),
            speed_limit_input: "1.0".to_string(),
        }
    }

    pub fn update_network_scan(&mut self) {
        if let Some(ref mut limiter) = self.network_limiter {
            match limiter.scan_network_processes() {
                Ok(()) => {
                    tracing::info!("✅ Scan réseau terminé - données temps réel");
                }
                Err(e) => {
                    tracing::error!("❌ Erreur scan réseau: {}", e);
                }
            }
        }
    }

    pub fn scan_network_processes(&mut self) {
        tracing::info!("🔄 Scan réseau demandé");
        self.update_network_scan();
    }

    pub fn limit_process(&mut self, pid: u32) {
        tracing::info!("🎯 Début limitation processus PID {}", pid);
        
        if let Some(ref mut limiter) = self.network_limiter {
            // Vérifier si le processus existe dans le scan
            let process_exists = limiter.get_processes().iter().any(|p| p.pid == pid);
            if !process_exists {
                tracing::warn!("⚠️ Processus PID {} non trouvé dans le scan réseau", pid);
                return;
            }
            
            let limit_mbps = match crate::network::parse_speed_limit_mbps(&self.speed_limit_input) {
                Ok(mbps) => {
                    tracing::info!("📊 Limitation parse: {} MB/s → OK", mbps);
                    mbps
                },
                Err(e) => {
                    tracing::error!("❌ Format de limitation invalide '{}': {}", self.speed_limit_input, e);
                    return;
                }
            };
            
            let limit_kbps = (limit_mbps * 1024.0) as u32;
            tracing::info!("🔢 Conversion: {:.1} MB/s → {} KB/s", limit_mbps, limit_kbps);
            
            match limiter.set_process_speed_limit(pid, limit_kbps) {
                Ok(()) => {
                    tracing::info!("✅ Limitation QoS appliquée: PID {} → {:.1} MB/s ({} KB/s)", pid, limit_mbps, limit_kbps);
                    
                    // Vérifier immédiatement si la politique a été créée
                    match limiter.verify_qos_policies() {
                        Ok(policies) => {
                            let policy_count = policies.len();
                            tracing::info!("📋 Vérification: {} politiques QoS trouvées après création", policy_count);
                            
                            // Chercher notre politique spécifique
                            let our_policy_name = format!("GameBooster_Limit_{}", pid);
                            let found = policies.iter().any(|p| p.name == our_policy_name);
                            if found {
                                tracing::info!("✅ Politique {} confirmée active", our_policy_name);
                            } else {
                                tracing::warn!("⚠️ Politique {} non trouvée dans la liste active", our_policy_name);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("⚠️ Impossible de vérifier les politiques: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Échec limitation QoS PID {}: {}", pid, e);
                }
            }
        } else {
            tracing::error!("❌ NetworkLimiter non initialisé pour PID {}", pid);
        }
    }

    pub fn remove_process_limit(&mut self, pid: u32) {
        if let Some(ref mut limiter) = self.network_limiter {
            match limiter.remove_process_limit(pid) {
                Ok(()) => {
                    tracing::info!("✅ Limitation supprimée: PID {}", pid);
                }
                Err(e) => {
                    tracing::error!("❌ Échec suppression limitation PID {}: {}", pid, e);
                }
            }
        }
    }

    pub fn apply_speed_limit_to_selected(&mut self) {
        let selected_pids: Vec<u32> = self.processes.iter().copied().collect();
        
        for pid in selected_pids {
            self.limit_process(pid);
        }
        
        if !self.processes.is_empty() {
            tracing::info!("✅ Limitation appliquée à {} processus sélectionnés", self.processes.len());
        }
    }

    pub fn select_all_processes(&mut self) {
        if let Some(ref limiter) = self.network_limiter {
            self.processes.clear();
            for process in limiter.get_processes() {
                self.processes.insert(process.pid);
            }
            tracing::info!("✅ {} processus sélectionnés", self.processes.len());
        }
    }

    pub fn deselect_all_processes(&mut self) {
        let count = self.processes.len();
        self.processes.clear();
        tracing::info!("✅ {} processus désélectionnés", count);
    }

    pub fn clear_all_network_limits(&mut self) {
        if let Some(ref mut limiter) = self.network_limiter {
            match limiter.clear_all_limits() {
                Ok(()) => {
                    tracing::info!("✅ Toutes les limitations supprimées");
                }
                Err(e) => {
                    tracing::error!("❌ Échec suppression globale: {}", e);
                }
            }
        }
    }
}

impl eframe::App for CleanRamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(self.theme.visuals.clone());
        let is_linux = self.windows_version_string.to_lowercase() == "linux";

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.active_tab == Tab::Memory, "🧠 Mémoire").clicked() {
                    self.active_tab = Tab::Memory;
                }
                
                let optimization_label = if is_linux { "⚙️ Optimisation (WIP)" } else { "⚙️ Optimisation" };
                if ui.selectable_label(self.active_tab == Tab::Optimization, optimization_label).clicked() {
                    self.active_tab = Tab::Optimization;
                }

                let services_label = if is_linux { "🛡️ Services (WIP)" } else { "🛡️ Services" };
                if ui.selectable_label(self.active_tab == Tab::Services, services_label).clicked() {
                    self.active_tab = Tab::Services;
                }

                // Le planificateur peut rester, il est multiplateforme en théorie
                if ui.selectable_label(self.active_tab == Tab::Scheduler, "⏰ Planificateur").clicked() {
                    self.active_tab = Tab::Scheduler;
                }

                let network_label = if is_linux { "📡 Réseau (WIP)" } else { "📡 Réseau" };
                if ui.selectable_label(self.active_tab == Tab::Network, network_label).clicked() { 
                    self.active_tab = Tab::Network;
                }
                if ui.selectable_label(self.active_tab == Tab::Settings, "⚙️ Paramètres").clicked() {
                    self.active_tab = Tab::Settings;
                }
            });

            ui.separator();

            let theme_clone = self.theme.clone();
            match self.active_tab {
                Tab::Memory => memory_ui::draw_memory_tab(self, ui, &theme_clone),
                Tab::Optimization => {
                    if is_linux {
                        ui.centered_and_justified(|ui| {
                            ui.label("Cet onglet est en cours de développement pour Linux.");
                        });
                    } else {
                        disk_ui::draw_disk_tab(self, ui);
                    }
                }
                Tab::Services => {
                    if is_linux {
                        ui.centered_and_justified(|ui| {
                            ui.label("Cet onglet est en cours de développement pour Linux.");
                        });
                    } else {
                        services_ui::services_ui(self, ui);
                    }
                }
                Tab::Scheduler => scheduler_ui::draw_scheduler_tab(self, ui),
                Tab::Network => {
                    if is_linux {
                        ui.centered_and_justified(|ui| {
                            ui.label("Cet onglet est en cours de développement pour Linux.");
                        });
                    } else {
                        network_ui::draw_network_tab(self, ui);
                    }
                }
                Tab::Settings => settings_ui::draw_settings_tab(self, ui),
            }
        });

        if self.is_first_frame {
            self.is_first_frame = false;
            // Pas de vérification automatique au lancement pour éviter l'ouverture de PowerShell
        }
    }
}