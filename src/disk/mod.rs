// Disk cleaning functionality
pub mod temp_files;
pub mod browser_cache;
pub mod thumbnails;

use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DiskCleaningOptions {
    pub clean_temp_files: bool,
    pub clean_browser_cache: bool,
    pub clean_thumbnails: bool,
    pub clean_recycle_bin: bool,
    pub clean_system_cache: bool,
    pub win10_optimizations: bool,
    pub win11_optimizations: bool,
}

impl Default for DiskCleaningOptions {
    fn default() -> Self {
        Self {
            clean_temp_files: true,
            clean_browser_cache: true,
            clean_thumbnails: true,
            clean_recycle_bin: false,
            clean_system_cache: false,
            win10_optimizations: false,
            win11_optimizations: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskCleaningResults {
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub total_space_freed: u64,
    pub temp_files_cleaned: u64,
    pub cache_cleaned: u64,
    pub thumbnails_cleaned: u64,
    pub files_processed: u32,
    pub errors: Vec<String>,
    pub is_completed: bool,
    pub duration: Option<std::time::Duration>,
}

impl DiskCleaningResults {
    pub fn new() -> Self {
        Self {
            start_time: Local::now(),
            end_time: None,
            total_space_freed: 0,
            temp_files_cleaned: 0,
            cache_cleaned: 0,
            thumbnails_cleaned: 0,
            files_processed: 0,
            errors: Vec::new(),
            is_completed: false,
            duration: None,
        }
    }

    pub fn complete(&mut self) {
        self.end_time = Some(Local::now());
        self.is_completed = true;
        if let Some(end) = self.end_time {
            self.duration = Some(std::time::Duration::from_millis(
                (end.timestamp_millis() - self.start_time.timestamp_millis()) as u64
            ));
        }
    }
}

pub async fn clean_disk_with_options(options: DiskCleaningOptions) -> Result<DiskCleaningResults> {
    let mut results = DiskCleaningResults::new();

    // Clean temporary files if selected
    if options.clean_temp_files {
        match temp_files::clean_temp_files().await {
            Ok(cleaned) => {
                results.temp_files_cleaned = cleaned;
                results.total_space_freed += cleaned;
                println!("Fichiers temporaires nettoyés: {} bytes", cleaned);
            }
            Err(e) => {
                results.errors.push(format!("Erreur nettoyage fichiers temporaires: {}", e));
                println!("Erreur lors du nettoyage des fichiers temporaires: {}", e);
            }
        }
    }

    // Clean browser cache if selected
    if options.clean_browser_cache {
        match browser_cache::clean_browser_cache().await {
            Ok(cleaned) => {
                results.cache_cleaned = cleaned;
                results.total_space_freed += cleaned;
                println!("Cache navigateur nettoyé: {} bytes", cleaned);
            }
            Err(e) => {
                results.errors.push(format!("Erreur nettoyage cache navigateur: {}", e));
                println!("Erreur lors du nettoyage du cache navigateur: {}", e);
            }
        }
    }

    // Clean thumbnails if selected
    if options.clean_thumbnails {
        match thumbnails::clean_thumbnails().await {
            Ok(cleaned) => {
                results.thumbnails_cleaned = cleaned;
                results.total_space_freed += cleaned;
                println!("Miniatures nettoyées: {} bytes", cleaned);
            }
            Err(e) => {
                results.errors.push(format!("Erreur nettoyage miniatures: {}", e));
                println!("Erreur lors du nettoyage des miniatures: {}", e);
            }
        }
    }

    // TODO: Ajouter support pour recycle_bin et system_cache quand options sélectionnées
    if options.clean_recycle_bin {
        println!("Nettoyage de la corbeille (non implémenté)");
    }
    
    if options.clean_system_cache {
        println!("Nettoyage du cache système (non implémenté)");
    }

    results.complete();
    println!("Nettoyage de disque terminé. Total libéré: {} bytes", results.total_space_freed);
    Ok(results)
}

pub async fn clean_disk() -> Result<DiskCleaningResults> {
    clean_disk_with_options(DiskCleaningOptions::default()).await
}

// Get disk cleaning preview without actually cleaning
pub fn get_disk_cleaning_preview() -> Result<DiskCleaningResults> {
    scan_disk_with_options(DiskCleaningOptions::default())
}

// Scan disk to get cleaning preview with options without actually cleaning
pub fn scan_disk_with_options(options: DiskCleaningOptions) -> Result<DiskCleaningResults> {
    let mut results = DiskCleaningResults::new();
    
    // Get size estimates without cleaning based on options
    if options.clean_temp_files {
        if let Ok(temp_size) = temp_files::get_temp_file_size() {
            results.temp_files_cleaned = temp_size;
            results.total_space_freed += temp_size;
        }
    }
    
    if options.clean_browser_cache {
        if let Ok(cache_size) = browser_cache::get_browser_cache_size() {
            results.cache_cleaned = cache_size;
            results.total_space_freed += cache_size;
        }
    }
    
    if options.clean_thumbnails {
        if let Ok(thumbnails_size) = thumbnails::get_thumbnails_size() {
            results.thumbnails_cleaned = thumbnails_size;
            results.total_space_freed += thumbnails_size;
        }
    }
    
    results.complete();
    Ok(results)
}
