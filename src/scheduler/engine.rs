// Task execution engine for scheduled operations
use super::{TaskScheduler, TaskType, ScheduledTask};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

pub struct TaskEngine {
    scheduler: Arc<Mutex<TaskScheduler>>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl TaskEngine {
    pub fn new(scheduler: TaskScheduler) -> Self {
        Self {
            scheduler: Arc::new(Mutex::new(scheduler)),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub async fn start(&self) {
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);
        
        {
            let mut scheduler = self.scheduler.lock().await;
            scheduler.start();
        }

        info!("Task engine started");

        // Main execution loop
        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            if let Err(e) = self.check_and_execute_tasks().await {
                error!("Error in task execution loop: {}", e);
            }

            // Check every 30 seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }

    pub async fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        
        {
            let mut scheduler = self.scheduler.lock().await;
            scheduler.stop();
        }

        info!("Task engine stopped");
    }

    async fn check_and_execute_tasks(&self) -> Result<()> {
        let pending_tasks = {
            let mut scheduler = self.scheduler.lock().await;
            scheduler.get_pending_tasks()
        };

        for task in pending_tasks {
            info!("Executing scheduled task: {} ({})", task.name, task.id);
            
            let result = self.execute_task(&task).await;
            
            let (success, error) = match result {
                Ok(_) => {
                    info!("Task completed successfully: {}", task.name);
                    (true, None)
                }
                Err(e) => {
                    error!("Task failed: {} - {}", task.name, e);
                    (false, Some(e.to_string()))
                }
            };

            // Update task status
            {
                let mut scheduler = self.scheduler.lock().await;
                scheduler.mark_task_completed(&task.id, success, error);
            }
        }

        Ok(())
    }

    async fn execute_task(&self, task: &ScheduledTask) -> Result<()> {
        match &task.task_type {
            TaskType::CleanRam { threshold_percentage } => {
                self.execute_ram_cleanup(*threshold_percentage).await
            }
            TaskType::CleanDisk { size_threshold_mb, options } => {
                self.execute_disk_cleanup(*size_threshold_mb, options.clone()).await
            }
            TaskType::DefenderToggle { enable } => {
                self.execute_defender_toggle(*enable).await
            }
        }
    }

    async fn execute_ram_cleanup(&self, threshold_percentage: u8) -> Result<()> {
        // Check if threshold is met
        let memory_info = crate::memory::get_detailed_system_memory_info();
        let usage_percent = ((memory_info.total_physical - memory_info.avail_physical) * 100) / memory_info.total_physical;
        
        if usage_percent as u8 >= threshold_percentage {
            info!("RAM usage {}% exceeds threshold {}%, cleaning...", usage_percent, threshold_percentage);
            
            // Execute RAM cleanup
            match crate::memory::clean_memory() {
                Ok(freed) => {
                    info!("RAM cleanup completed, freed {} MB", freed.total_freed() / 1024 / 1024);
                    Ok(())
                }
                Err(e) => {
                    error!("RAM cleanup failed: {}", e);
                    Err(e)
                }
            }
        } else {
            info!("RAM usage {}% is below threshold {}%, skipping cleanup", usage_percent, threshold_percentage);
            Ok(())
        }
    }

    async fn execute_disk_cleanup(&self, size_threshold_mb: u64, options: crate::disk::DiskCleaningOptions) -> Result<()> {
        // First scan to check if threshold is met
        let scan_result = crate::disk::scan_disk_with_options_sync(options.clone())?;
        let potential_mb = scan_result.total_space_to_free / 1024 / 1024;
        
        if potential_mb >= size_threshold_mb {
            info!("Potential disk cleanup {} MB exceeds threshold {} MB, cleaning...", potential_mb, size_threshold_mb);
            
            // Execute disk cleanup with specific options
            match crate::disk::clean_disk_with_options_sync(options) {
                Ok(result) => {
                    let freed_mb = result.total_space_freed / 1024 / 1024;
                    info!("Disk cleanup completed, freed {} MB", freed_mb);
                    info!("- Temp files: {} MB", result.temp_files_freed / 1024 / 1024);
                    info!("- Browser cache: {} MB", result.cache_freed / 1024 / 1024);
                    info!("- Thumbnails: {} MB", result.thumbnails_freed / 1024 / 1024);
                    info!("- Recycle bin: {} MB", result.recycle_bin_freed / 1024 / 1024);
                    Ok(())
                }
                Err(e) => {
                    error!("Disk cleanup failed: {}", e);
                    Err(e)
                }
            }
        } else {
            info!("Potential disk cleanup {} MB is below threshold {} MB, skipping cleanup", potential_mb, size_threshold_mb);
            Ok(())
        }
    }

    async fn execute_defender_toggle(&self, enable: bool) -> Result<()> {
        if enable {
            info!("Enabling Windows Defender...");
            match crate::services::defender::DefenderService::enable_immediately() {
                Ok(_) => {
                    info!("Windows Defender enabled successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to enable Windows Defender: {}", e);
                    Err(e)
                }
            }
        } else {
            info!("Disabling Windows Defender...");
            match crate::services::defender::DefenderService::disable_immediately() {
                Ok(_) => {
                    info!("Windows Defender disabled successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to disable Windows Defender: {}", e);
                    Err(e)
                }
            }
        }
    }

    pub async fn get_scheduler(&self) -> Arc<Mutex<TaskScheduler>> {
        self.scheduler.clone()
    }
}
