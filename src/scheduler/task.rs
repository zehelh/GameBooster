// Task execution logic for scheduler
// This module contains helper functions for task execution

use crate::memory::clean_memory;
use crate::disk::DiskCleaningOptions;
use crate::scheduler::{TaskType, ScheduledTask};
use anyhow::Result;
use chrono::Local;

pub async fn execute_task(task: &ScheduledTask) -> Result<String> {
    match &task.task_type {
        TaskType::CleanRam { threshold_percentage } => {
            execute_ram_cleaning(*threshold_percentage).await
        }
        TaskType::CleanDisk { size_threshold_mb, options } => {
            execute_disk_cleaning(*size_threshold_mb, options.clone()).await
        }
        TaskType::DefenderToggle { enable } => {
            execute_defender_toggle(*enable).await
        }
    }
}

async fn execute_ram_cleaning(threshold_percentage: u8) -> Result<String> {
    // Check if threshold is met
    let memory_info = crate::memory::get_detailed_system_memory_info();
    let usage_percent = ((memory_info.total_physical - memory_info.avail_physical) * 100) / memory_info.total_physical;
    
    if usage_percent as u8 >= threshold_percentage {
        match clean_memory() {
            Ok(results) => {
                let freed = results.total_freed();
                Ok(format!("RAM cleaning completed. Freed: {} bytes ({}% usage exceeded threshold {}%)", 
                    freed, usage_percent, threshold_percentage))
            }
            Err(e) => Err(anyhow::anyhow!("RAM cleaning failed: {}", e)),
        }
    } else {
        Ok(format!("RAM usage {}% is below threshold {}%, skipping cleanup", 
            usage_percent, threshold_percentage))
    }
}

async fn execute_disk_cleaning(size_threshold_mb: u64, options: DiskCleaningOptions) -> Result<String> {
    // Check if threshold is met
    let scan_result = crate::disk::scan_disk_with_options_sync(options.clone())?;
    let potential_mb = scan_result.total_space_to_free / 1024 / 1024;
    
    if potential_mb >= size_threshold_mb {
        match crate::disk::clean_disk_with_options_sync(options) {
            Ok(result) => {
                let freed_mb = result.total_space_freed / 1024 / 1024;
                let details = format!(
                    "Disk cleaning completed. Freed: {} MB (potential {} MB exceeded threshold {} MB)\n\
                     - Temp files: {} MB\n\
                     - Browser cache: {} MB\n\
                     - Thumbnails: {} MB\n\
                     - Recycle bin: {} MB", 
                    freed_mb, potential_mb, size_threshold_mb,
                    result.temp_files_freed / 1024 / 1024,
                    result.cache_freed / 1024 / 1024,
                    result.thumbnails_freed / 1024 / 1024,
                    result.recycle_bin_freed / 1024 / 1024
                );
                Ok(details)
            }
            Err(e) => Err(anyhow::anyhow!("Disk cleaning failed: {}", e)),
        }
    } else {
        Ok(format!("Potential disk cleanup {} MB is below threshold {} MB, skipping cleanup", 
            potential_mb, size_threshold_mb))
    }
}

async fn execute_defender_toggle(enable: bool) -> Result<String> {
    if enable {
        match crate::services::defender::DefenderService::enable_immediately() {
            Ok(_) => Ok("Windows Defender enabled successfully".to_string()),
            Err(e) => Err(anyhow::anyhow!("Failed to enable Windows Defender: {}", e)),
        }
    } else {
        match crate::services::defender::DefenderService::disable_immediately() {
            Ok(_) => Ok("Windows Defender disabled successfully".to_string()),
            Err(e) => Err(anyhow::anyhow!("Failed to disable Windows Defender: {}", e)),
        }
    }
}

pub fn is_task_due(task: &ScheduledTask) -> bool {
    if !task.enabled {
        return false;
    }

    match &task.next_run {
        Some(next_run) => Local::now() >= *next_run,
        None => true, // First run
    }
}
