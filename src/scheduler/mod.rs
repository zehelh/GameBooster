// Internal task scheduler module for automatic cleaning and system management
pub mod task;
pub mod config;
pub mod engine;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, NaiveTime, Weekday, Duration, TimeZone, Datelike};
use std::collections::HashMap;
use anyhow::Result;
use crate::disk::DiskCleaningOptions;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    CleanRam { threshold_percentage: u8 }, // Clean RAM when usage exceeds threshold
    CleanDisk { 
        size_threshold_mb: u64,
        options: DiskCleaningOptions,
    },  // Clean disk with specific options when potential savings exceed threshold
    DefenderToggle { enable: bool },       // Enable/disable Windows Defender
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScheduleRule {
    OnStartup,                              // Run once at application startup
    Interval { minutes: u32 },              // Every X minutes
    Daily { time: NaiveTime },              // Daily at specific time
    Weekly { weekday: Weekday, time: NaiveTime }, // Weekly on specific day and time
    OnCondition,                            // Run when condition is met (for thresholds)
}

// Configuration for auto-startup with Windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoStartupConfig {
    pub enabled: bool,
    pub start_minimized: bool,
    pub auto_start_scheduler: bool,
    pub startup_delay_seconds: u32,
}

impl Default for AutoStartupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            start_minimized: true,
            auto_start_scheduler: true,
            startup_delay_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,                       // Human-readable name
    pub description: String,                // Task description
    pub task_type: TaskType,
    pub schedule: ScheduleRule,
    pub enabled: bool,
    pub last_run: Option<DateTime<Local>>,
    pub next_run: Option<DateTime<Local>>,
    pub run_count: u32,
    pub success_count: u32,
    pub last_error: Option<String>,
}

impl ScheduledTask {
    pub fn new(id: String, name: String, description: String, task_type: TaskType, schedule: ScheduleRule) -> Self {
        Self {
            id,
            name,
            description,
            task_type,
            schedule,
            enabled: true,
            last_run: None,
            next_run: None,
            run_count: 0,
            success_count: 0,
            last_error: None,
        }
    }
}

pub struct TaskScheduler {
    tasks: HashMap<String, ScheduledTask>,
    config_path: String,
    running: bool,
}

impl TaskScheduler {
    pub fn new(config_path: &str) -> Self {
        Self {
            tasks: HashMap::new(),
            config_path: config_path.to_string(),
            running: false,
        }
    }

    pub fn load_tasks(&mut self) -> Result<()> {
        if std::path::Path::new(&self.config_path).exists() {
            let content = std::fs::read_to_string(&self.config_path)?;
            self.tasks = serde_json::from_str(&content)?;
            self.update_next_run_times();
        }
        Ok(())
    }

    pub fn save_tasks(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.tasks)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn add_task(&mut self, mut task: ScheduledTask) {
        task.next_run = self.calculate_next_run(&task);
        self.tasks.insert(task.id.clone(), task);
        let _ = self.save_tasks();
    }

    pub fn remove_task(&mut self, task_id: &str) -> bool {
        if self.tasks.remove(task_id).is_some() {
            let _ = self.save_tasks();
            true
        } else {
            false
        }
    }

    pub fn get_task(&self, task_id: &str) -> Option<&ScheduledTask> {
        self.tasks.get(task_id)
    }

    pub fn get_all_tasks(&self) -> Vec<&ScheduledTask> {
        self.tasks.values().collect()
    }

    pub fn update_task(&mut self, task: ScheduledTask) {
        if self.tasks.contains_key(&task.id) {
            let mut updated_task = task.clone();
            updated_task.next_run = self.calculate_next_run(&updated_task);
            self.tasks.insert(task.id.clone(), updated_task);
            let _ = self.save_tasks();
        }
    }

    pub fn get_pending_tasks(&mut self) -> Vec<ScheduledTask> {
        let now = Local::now();
        let mut pending = Vec::new();
        
        let tasks_to_check: Vec<_> = self.tasks.iter().map(|(id, task)| (id.clone(), task.clone())).collect();
        
        for (_task_id, task) in tasks_to_check {
            if !task.enabled {
                continue;
            }

            let should_run = match &task.schedule {
                ScheduleRule::OnStartup => task.last_run.is_none(),
                ScheduleRule::OnCondition => self.check_condition_met(&task),
                _ => {
                    if let Some(next_run) = task.next_run {
                        now >= next_run
                    } else {
                        false
                    }
                }
            };

            if should_run {
                pending.push(task);
            }
        }

        pending
    }

    pub fn mark_task_completed(&mut self, task_id: &str, success: bool, error: Option<String>) {
        // First, calculate next run before borrowing mutably
        let next_run = if let Some(task) = self.tasks.get(task_id) {
            let mut task_clone = task.clone();
            task_clone.last_run = Some(Local::now());
            task_clone.run_count += 1;
            if success {
                task_clone.success_count += 1;
                task_clone.last_error = None;
            } else {
                task_clone.last_error = error.clone();
            }
            self.calculate_next_run(&task_clone)
        } else {
            None
        };

        // Now update the actual task
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.last_run = Some(Local::now());
            task.run_count += 1;
            if success {
                task.success_count += 1;
                task.last_error = None;
            } else {
                task.last_error = error;
            }
            task.next_run = next_run;
            let _ = self.save_tasks();
        }
    }

    fn check_condition_met(&self, task: &ScheduledTask) -> bool {
        match &task.task_type {
            TaskType::CleanRam { threshold_percentage } => {
                // Check if RAM usage exceeds threshold
                let memory_info = crate::memory::get_detailed_system_memory_info();
                let usage_percent = ((memory_info.total_physical - memory_info.avail_physical) * 100) / memory_info.total_physical;
                usage_percent as u8 >= *threshold_percentage
            }
            TaskType::CleanDisk { size_threshold_mb, options } => {
                // Check if potential disk cleanup exceeds threshold
                if let Ok(scan_result) = crate::disk::scan_disk_with_options_sync(options.clone()) {
                    let potential_mb = scan_result.total_space_to_free / 1024 / 1024;
                    potential_mb >= *size_threshold_mb
                } else {
                    false
                }
            }
            TaskType::DefenderToggle { .. } => false, // Defender toggle is manual only
        }
    }

    fn update_next_run_times(&mut self) {
        let task_ids: Vec<_> = self.tasks.keys().cloned().collect();
        for task_id in task_ids {
            if let Some(task) = self.tasks.get(&task_id) {
                let next_run = self.calculate_next_run(task);
                if let Some(task) = self.tasks.get_mut(&task_id) {
                    task.next_run = next_run;
                }
            }
        }
    }

    pub fn calculate_next_run(&self, task: &ScheduledTask) -> Option<DateTime<Local>> {
        match &task.schedule {
            ScheduleRule::OnStartup | ScheduleRule::OnCondition => None,
            ScheduleRule::Interval { minutes } => {
                let now = Local::now();
                Some(now + Duration::minutes(*minutes as i64))
            }
            ScheduleRule::Daily { time } => {
                let now = Local::now();
                let today = now.date_naive();
                let target_datetime = today.and_time(*time);
                let target_local = Local.from_local_datetime(&target_datetime).earliest()?;
                
                if target_local > now {
                    Some(target_local)
                } else {
                    // Schedule for tomorrow
                    let tomorrow = today + Duration::days(1);
                    let tomorrow_target = tomorrow.and_time(*time);
                    Local.from_local_datetime(&tomorrow_target).earliest()
                }
            }
            ScheduleRule::Weekly { weekday, time } => {
                let now = Local::now();
                let today = now.date_naive();
                let current_weekday = today.weekday();
                
                let days_until_target = if *weekday == current_weekday {
                    // Check if time has already passed today
                    let target_datetime = today.and_time(*time);
                    let target_local = Local.from_local_datetime(&target_datetime).earliest()?;
                    
                    if target_local > now {
                        0
                    } else {
                        7
                    }
                } else {
                    let target_num = weekday.num_days_from_monday();
                    let current_num = current_weekday.num_days_from_monday();
                    
                    if target_num > current_num {
                        target_num - current_num
                    } else {
                        7 - (current_num - target_num)
                    }
                };
                
                let target_date = today + Duration::days(days_until_target as i64);
                let target_datetime = target_date.and_time(*time);
                Local.from_local_datetime(&target_datetime).earliest()
            }
        }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    // Auto-startup management functions
    pub fn load_startup_config() -> AutoStartupConfig {
        let config_path = std::env::current_exe()
            .map(|exe| exe.parent().unwrap_or(&std::path::Path::new(".")).join("startup_config.json"))
            .unwrap_or_else(|_| std::path::PathBuf::from("startup_config.json"));
        
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<AutoStartupConfig>(&content) {
                    return config;
                }
            }
        }
        AutoStartupConfig::default()
    }

    pub fn save_startup_config(config: &AutoStartupConfig) -> Result<()> {
        let config_path = std::env::current_exe()
            .map(|exe| exe.parent().unwrap_or(&std::path::Path::new(".")).join("startup_config.json"))
            .unwrap_or_else(|_| std::path::PathBuf::from("startup_config.json"));
        
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn set_windows_startup(enable: bool, start_minimized: bool) -> Result<()> {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu.open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_ALL_ACCESS)?;
        
        let app_name = "GameBooster";
        
        if enable {
            let exe_path = std::env::current_exe()?;
            let mut command = format!("\"{}\"", exe_path.display());
            
            if start_minimized {
                command.push_str(" --minimized");
            }
            
            run_key.set_value(app_name, &command)?;
            tracing::info!("✅ Auto-startup enabled: {}", command);
        } else {
            if let Err(e) = run_key.delete_value(app_name) {
                // Ignore error if value doesn't exist
                tracing::warn!("Auto-startup entry removal: {}", e);
            } else {
                tracing::info!("✅ Auto-startup disabled");
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn set_windows_startup(_enable: bool, _start_minimized: bool) -> Result<()> {
        Err(anyhow::anyhow!("Auto-startup is only supported on Windows"))
    }

    pub fn check_startup_args() -> (bool, bool) {
        let args: Vec<String> = std::env::args().collect();
        let is_minimized = args.iter().any(|arg| arg == "--minimized");
        let should_auto_start_scheduler = Self::load_startup_config().auto_start_scheduler;
        
        (is_minimized, should_auto_start_scheduler)
    }
}

// Default task templates
impl TaskScheduler {
    pub fn create_default_ram_cleanup_task() -> ScheduledTask {
        ScheduledTask::new(
            "ram_cleanup_threshold".to_string(),
            "RAM Cleanup (Threshold)".to_string(),
            "Automatically clean RAM when usage exceeds 85%".to_string(),
            TaskType::CleanRam { threshold_percentage: 85 },
            ScheduleRule::OnCondition,
        )
    }

    pub fn create_default_disk_cleanup_task() -> ScheduledTask {
        ScheduledTask::new(
            "disk_cleanup_daily".to_string(),
            "Daily Disk Cleanup".to_string(),
            "Clean temporary files and cache daily at 2:00 AM".to_string(),
            TaskType::CleanDisk { size_threshold_mb: 100, options: Default::default() },
            ScheduleRule::Daily { time: NaiveTime::from_hms_opt(2, 0, 0).unwrap() },
        )
    }

    pub fn create_default_defender_disable_task() -> ScheduledTask {
        ScheduledTask::new(
            "defender_disable_weekly".to_string(),
            "Weekly Defender Disable".to_string(),
            "Disable Windows Defender every Monday at 9:00 AM".to_string(),
            TaskType::DefenderToggle { enable: false },
            ScheduleRule::Weekly { 
                weekday: Weekday::Mon, 
                time: NaiveTime::from_hms_opt(9, 0, 0).unwrap() 
            },
        )
    }
}
