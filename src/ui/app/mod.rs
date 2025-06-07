use std::collections::HashSet;

use crate::disk::{DiskCleaningOptions, DiskCleaningResults};
use crate::memory::CleaningResults;
use crate::services::defender::DefenderStatus;

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
    Hdd,
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
        
        // Network manager initialization commented out for simplification

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
            windows_version_string: format!("Windows {}", env!("CARGO_PKG_VERSION")),
            logo: dummy_texture_id,
            ram_icon: dummy_texture_id,
            is_first_frame: true,
        }
    }
}

impl eframe::App for CleanRamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(self.theme.visuals.clone());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.active_tab == Tab::Memory, "🧠 Mémoire").clicked() {
                    self.active_tab = Tab::Memory;
                }
                if ui.selectable_label(self.active_tab == Tab::Hdd, "💾 Disque").clicked() {
                    self.active_tab = Tab::Hdd;
                }
                if ui.selectable_label(self.active_tab == Tab::Services, "🛡️ Services").clicked() {
                    self.active_tab = Tab::Services;
                }
                if ui.selectable_label(self.active_tab == Tab::Scheduler, "⏰ Planificateur").clicked() {
                    self.active_tab = Tab::Scheduler;
                }
                if ui.selectable_label(self.active_tab == Tab::Network, "🌐 Réseau").clicked() {
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
                Tab::Hdd => disk_ui::draw_disk_tab(self, ui),
                Tab::Services => services_ui::services_ui(self, ui),
                Tab::Scheduler => scheduler_ui::draw_scheduler_tab(self, ui),
                Tab::Network => network_ui::draw_network_tab(self, ui),
                Tab::Settings => settings_ui::draw_settings_tab(self, ui),
            }
        });

        if self.is_first_frame {
            self.is_first_frame = false;
            // Pas de vérification automatique au lancement pour éviter l'ouverture de PowerShell
        }
    }
} 