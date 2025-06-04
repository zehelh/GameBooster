// Task execution logic for scheduler

use crate::memory::clean_memory;
use crate::scheduler::{TaskType, ScheduledTask};
use chrono::Local;
use anyhow::Result;

pub async fn execute_task(task: &ScheduledTask) -> Result<String> {
    match task.task_type {
        TaskType::CleanRam => execute_ram_cleaning().await,
        TaskType::CleanDisk => execute_disk_cleaning().await,
        TaskType::OptimizeServices => execute_service_optimization().await,
        TaskType::NetworkLimit => execute_network_limiting().await,
    }
}

async fn execute_ram_cleaning() -> Result<String> {
    match clean_memory() {
        Ok(results) => {
            let freed = results.total_freed();
            Ok(format!("RAM cleaning completed. Freed: {} bytes", freed))
        }
        Err(e) => Err(e),
    }
}

async fn execute_disk_cleaning() -> Result<String> {
    // TODO: Implement disk cleaning
    Ok("Disk cleaning not yet implemented".to_string())
}

async fn execute_service_optimization() -> Result<String> {
    // TODO: Implement service optimization
    Ok("Service optimization not yet implemented".to_string())
}

async fn execute_network_limiting() -> Result<String> {
    // TODO: Implement network limiting
    Ok("Network limiting not yet implemented".to_string())
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
