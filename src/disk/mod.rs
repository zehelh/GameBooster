// Disk cleaning functionality
pub mod temp_files;
pub mod browser_cache;
pub mod thumbnails;

use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

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
        }
    }

    pub fn complete(&mut self) {
        self.end_time = Some(Local::now());
        self.is_completed = true;
    }
}

pub async fn clean_disk() -> Result<DiskCleaningResults> {
    let mut results = DiskCleaningResults::new();

    // Clean temporary files
    match temp_files::clean_temp_files().await {
        Ok(cleaned) => {
            results.temp_files_cleaned = cleaned;
            results.total_space_freed += cleaned;
        }
        Err(e) => results.errors.push(format!("Temp files: {}", e)),
    }

    // Clean browser cache
    match browser_cache::clean_browser_cache().await {
        Ok(cleaned) => {
            results.cache_cleaned = cleaned;
            results.total_space_freed += cleaned;
        }
        Err(e) => results.errors.push(format!("Browser cache: {}", e)),
    }

    // Clean thumbnails
    match thumbnails::clean_thumbnails().await {
        Ok(cleaned) => {
            results.thumbnails_cleaned = cleaned;
            results.total_space_freed += cleaned;
        }
        Err(e) => results.errors.push(format!("Thumbnails: {}", e)),
    }

    results.complete();
    Ok(results)
}
