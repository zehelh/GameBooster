// Disk cleaning functionality
pub mod temp_files;
pub mod browser_cache;
pub mod thumbnails;
pub mod recycle_bin;

use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiskCleaningOptions {
    pub clean_temp_files: bool,
    pub clean_browser_cache: bool,
    pub clean_thumbnails: bool,
    pub clean_recycle_bin: bool,
    pub clean_system_cache: bool,
}

impl Default for DiskCleaningOptions {
    fn default() -> Self {
        Self {
            clean_temp_files: true,
            clean_browser_cache: true,
            clean_thumbnails: true,
            clean_recycle_bin: false,
            clean_system_cache: false,
        }
    }
}

// Represents the result of a disk scan (preview)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskScanResult {
    pub total_space_to_free: u64,
    pub temp_files_size: u64,
    pub cache_size: u64,
    pub thumbnails_size: u64,
    pub recycle_bin_size: u64,
    pub errors: Vec<String>,
    pub duration: Option<std::time::Duration>,
}

// Represents the result of a disk cleaning operation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskCleanResult {
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub total_space_freed: u64,
    pub temp_files_freed: u64,
    pub cache_freed: u64,
    pub thumbnails_freed: u64,
    pub recycle_bin_freed: u64,
    pub errors: Vec<String>,
    pub duration: Option<std::time::Duration>,
}

impl DiskCleanResult {
    pub fn new() -> Self {
        Self {
            start_time: Local::now(),
            ..Default::default()
        }
    }

    pub fn complete(&mut self) {
        self.end_time = Some(Local::now());
        if let Some(end) = self.end_time {
            self.duration = Some(std::time::Duration::from_millis(
                (end.timestamp_millis() - self.start_time.timestamp_millis()) as u64
            ));
        }
    }
}

pub async fn clean_disk_with_options(options: DiskCleaningOptions) -> Result<DiskCleanResult> {
    let mut results = DiskCleanResult::new();

    // Clean temporary files if selected
    if options.clean_temp_files {
        match temp_files::clean_temp_files().await {
            Ok(cleaned) => {
                results.temp_files_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Temporary files cleaned: {} bytes", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error cleaning temporary files: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }

    // Clean browser cache if selected
    if options.clean_browser_cache {
        match browser_cache::clean_browser_cache().await {
            Ok(cleaned) => {
                results.cache_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Browser cache cleaned: {} bytes", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error cleaning browser cache: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }

    // Clean thumbnails if selected
    if options.clean_thumbnails {
        match thumbnails::clean_thumbnails().await {
            Ok(cleaned) => {
                results.thumbnails_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Thumbnails cleaned: {} bytes", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error cleaning thumbnails: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }

    // Empty recycle bin if selected
    if options.clean_recycle_bin {
        match recycle_bin::empty_recycle_bin().await {
            Ok(cleaned) => {
                results.recycle_bin_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Recycle bin emptied, freed {} bytes.", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error emptying recycle bin: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }
    
    if options.clean_system_cache {
        tracing::warn!("System cache cleaning is not yet implemented.");
    }

    results.complete();
    tracing::info!("Disk cleaning finished. Total freed: {} bytes", results.total_space_freed);
    Ok(results)
}

// Scan disk to get cleaning preview with options without actually cleaning
pub async fn scan_disk_with_options(options: DiskCleaningOptions) -> Result<DiskScanResult> {
    let start_time = Local::now();
    let mut results = DiskScanResult::default();
    
    // Get size estimates without cleaning based on options
    if options.clean_temp_files {
        if let Ok(temp_size) = temp_files::get_temp_file_size_async().await {
            results.temp_files_size = temp_size;
            results.total_space_to_free += temp_size;
        }
    }
    
    if options.clean_browser_cache {
        if let Ok(cache_size) = browser_cache::get_browser_cache_size_async().await {
            results.cache_size = cache_size;
            results.total_space_to_free += cache_size;
        }
    }
    
    if options.clean_thumbnails {
        if let Ok(thumbnails_size) = thumbnails::get_thumbnails_size_async().await {
            results.thumbnails_size = thumbnails_size;
            results.total_space_to_free += thumbnails_size;
        }
    }

    if options.clean_recycle_bin {
        // Use sync version for scan to avoid PowerShell window
        if let Ok(recycle_bin_size) = recycle_bin::get_recycle_bin_size_sync() {
            results.recycle_bin_size = recycle_bin_size;
            results.total_space_to_free += recycle_bin_size;
        }
    }
    
    let end_time = Local::now();
    results.duration = Some(std::time::Duration::from_millis(
        (end_time.timestamp_millis() - start_time.timestamp_millis()) as u64
    ));
    Ok(results)
}

// Synchronous version of scan for UI that doesn't show PowerShell
pub fn scan_disk_with_options_sync(options: DiskCleaningOptions) -> Result<DiskScanResult> {
    let start_time = Local::now();
    let mut results = DiskScanResult::default();
    
    if options.clean_temp_files {
        if let Ok(temp_size) = temp_files::get_temp_file_size() {
            results.temp_files_size = temp_size;
            results.total_space_to_free += temp_size;
        }
    }
    
    if options.clean_browser_cache {
        if let Ok(cache_size) = browser_cache::get_browser_cache_size() {
            results.cache_size = cache_size;
            results.total_space_to_free += cache_size;
        }
    }
    
    if options.clean_thumbnails {
        if let Ok(thumbnails_size) = thumbnails::get_thumbnails_size() {
            results.thumbnails_size = thumbnails_size;
            results.total_space_to_free += thumbnails_size;
        }
    }

    if options.clean_recycle_bin {
        // Use sync version for scan to avoid PowerShell window
        if let Ok(recycle_bin_size) = recycle_bin::get_recycle_bin_size_sync() {
            results.recycle_bin_size = recycle_bin_size;
            results.total_space_to_free += recycle_bin_size;
        }
    }
    
    let end_time = Local::now();
    results.duration = Some(std::time::Duration::from_millis(
        (end_time.timestamp_millis() - start_time.timestamp_millis()) as u64
    ));
    Ok(results)
}

pub fn clean_disk_with_options_threaded(options: DiskCleaningOptions) -> Result<DiskCleanResult> {
    // Completely synchronous version that avoids PowerShell windows
    let mut results = DiskCleanResult::new();

    // Clean temporary files if selected
    if options.clean_temp_files {
        match temp_files::clean_temp_files_sync() {
            Ok(cleaned) => {
                results.temp_files_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Temporary files cleaned: {} bytes", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error cleaning temporary files: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }

    // Clean browser cache if selected
    if options.clean_browser_cache {
        match browser_cache::clean_browser_cache_sync() {
            Ok(cleaned) => {
                results.cache_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Browser cache cleaned: {} bytes", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error cleaning browser cache: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }

    // Clean thumbnails if selected
    if options.clean_thumbnails {
        match thumbnails::clean_thumbnails_sync() {
            Ok(cleaned) => {
                results.thumbnails_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Thumbnails cleaned: {} bytes", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error cleaning thumbnails: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }

    // Empty recycle bin if selected (using sync version that avoids PowerShell)
    if options.clean_recycle_bin {
        match recycle_bin::empty_recycle_bin_sync() {
            Ok(cleaned) => {
                results.recycle_bin_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Recycle bin emptied, freed {} bytes.", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error emptying recycle bin: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }
    
    if options.clean_system_cache {
        tracing::warn!("System cache cleaning is not yet implemented.");
    }

    results.complete();
    tracing::info!("Disk cleaning finished. Total freed: {} bytes", results.total_space_freed);
    Ok(results)
}

// Synchronous version of clean_disk_with_options for scheduler
pub fn clean_disk_with_options_sync(options: DiskCleaningOptions) -> Result<DiskCleanResult> {
    let mut results = DiskCleanResult::new();

    // Clean temporary files if selected
    if options.clean_temp_files {
        match temp_files::clean_temp_files_sync() {
            Ok(cleaned) => {
                results.temp_files_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Temporary files cleaned: {} bytes", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error cleaning temporary files: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }

    // Clean browser cache if selected
    if options.clean_browser_cache {
        match browser_cache::clean_browser_cache_sync() {
            Ok(cleaned) => {
                results.cache_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Browser cache cleaned: {} bytes", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error cleaning browser cache: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }

    // Clean thumbnails if selected
    if options.clean_thumbnails {
        match thumbnails::clean_thumbnails_sync() {
            Ok(cleaned) => {
                results.thumbnails_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Thumbnails cleaned: {} bytes", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error cleaning thumbnails: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }

    // Clean recycle bin if selected
    if options.clean_recycle_bin {
        match recycle_bin::empty_recycle_bin_sync() {
            Ok(cleaned) => {
                results.recycle_bin_freed = cleaned;
                results.total_space_freed += cleaned;
                tracing::info!("Recycle bin cleaned: {} bytes", cleaned);
            }
            Err(e) => {
                let error_msg = format!("Error cleaning recycle bin: {}", e);
                results.errors.push(error_msg.clone());
                tracing::error!("{}", error_msg);
            }
        }
    }

    results.complete();
    Ok(results)
}
