use std::collections::{HashSet, HashMap};

use crate::disk::{DiskCleaningOptions, DiskScanResult, DiskCleanResult};
use crate::memory::CleaningResults;
use crate::services::defender::DefenderStatus;
use crate::network::NetworkLimiter;
use crate::scheduler::{ScheduledTask, TaskScheduler};

use eframe::egui;
// use image::load_from_memory; // Temporarily disabled to prevent crashes
use poll_promise::Promise;
use chrono::{DateTime, Local, TimeZone};

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
    pub disk_scan_promise: Option<Promise<DiskScanResult>>,
    pub disk_clean_promise: Option<Promise<DiskCleanResult>>,
    pub last_disk_scan_result: Option<DiskScanResult>,
    pub last_disk_clean_result: Option<DiskCleanResult>,
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
    
    // Scheduler fields
    pub task_scheduler: TaskScheduler,
    pub scheduled_tasks: HashMap<String, ScheduledTask>,
    pub scheduler_running: bool,
    pub show_add_task_dialog: bool,
    pub editing_task_id: Option<String>,
    pub task_to_delete: Option<String>,
    pub last_scheduler_check: DateTime<Local>,
    
    // New task dialog fields
    pub new_task_name: String,
    pub new_task_description: String,
    pub new_task_type: usize,
    pub new_task_ram_threshold: u8,
    pub new_task_disk_threshold: u32,
    pub new_task_defender_action: bool,
    pub new_task_schedule_type: usize,
    pub new_task_interval_minutes: u32,
    pub new_task_daily_hour: u32,
    pub new_task_daily_minute: u32,
    pub new_task_weekly_day: usize,
    pub new_task_weekly_hour: u32,
    pub new_task_weekly_minute: u32,
    
    // Edit task dialog fields
    pub edit_task_name: String,
    pub edit_task_description: String,
    pub edit_task_type: usize,
    pub edit_task_ram_threshold: u8,
    pub edit_task_disk_threshold: u32,
    pub edit_task_defender_action: bool,
    pub edit_task_schedule_type: usize,
    pub edit_task_interval_minutes: u32,
    pub edit_task_daily_hour: u32,
    pub edit_task_daily_minute: u32,
    pub edit_task_weekly_day: usize,
    pub edit_task_weekly_hour: u32,
    pub edit_task_weekly_minute: u32,
    pub edit_disk_options: DiskCleaningOptions,
}

impl CleanRamApp {
    pub fn is_not_busy(&self) -> bool {
        // Only block UI during heavy operations, not status checks
        self.cleaning_promise.is_none() 
            && self.disk_scan_promise.is_none() 
            && self.disk_clean_promise.is_none() 
            && self.defender_action_promise.is_none()
    }

    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Create simple textures without loading images to prevent crashes
        let dummy_texture_id = egui::TextureId::default();
        
        // Network manager initialization
        let network_limiter = match crate::network::NetworkLimiter::new() {
            Ok(limiter) => {
                tracing::info!("✅ QoS network manager initialized");
                Some(limiter)
            }
            Err(e) => {
                tracing::error!("❌ Network manager initialization failed: {}", e);
                None
            }
        };

        // Initialize task scheduler and load existing tasks
        let mut task_scheduler = TaskScheduler::new("scheduled_tasks.json");
        let mut scheduled_tasks = HashMap::new();
        
        // Load existing tasks
        if let Err(e) = task_scheduler.load_tasks() {
            tracing::warn!("⚠️ Failed to load scheduled tasks: {}", e);
        } else {
            // Copy tasks from scheduler to app
            for task in task_scheduler.get_all_tasks() {
                scheduled_tasks.insert(task.id.clone(), task.clone());
            }
            tracing::info!("📋 Loaded {} scheduled tasks", scheduled_tasks.len());
        }

        Self {
            active_tab: Tab::Memory,
            theme: theme::dark_theme(),
            ram_usage: 0.0,
            cleaning_promise: None,
            last_cleaned_results: None,
            disk_options: DiskCleaningOptions::default(),
            disk_scan_promise: None,
            disk_clean_promise: None,
            last_disk_scan_result: None,
            last_disk_clean_result: None,
            processes: HashSet::new(),
            defender_status_promise: None,
            defender_action_promise: None,
            last_defender_status: None,
            windows_version_string: format!("Windows {}", env!("CARGO_PKG_VERSION")),
            logo: dummy_texture_id,
            ram_icon: dummy_texture_id,
            is_first_frame: true,
            network_limiter,
            process_search_text: String::new(),
            speed_limit_input: "1.0".to_string(),
            
            // Initialize scheduler fields
            task_scheduler,
            scheduled_tasks,
            scheduler_running: false,
            show_add_task_dialog: false,
            editing_task_id: None,
            task_to_delete: None,
            last_scheduler_check: Local::now(),
            
            // Initialize new task dialog fields
            new_task_name: String::new(),
            new_task_description: String::new(),
            new_task_type: 0,
            new_task_ram_threshold: 85,
            new_task_disk_threshold: 100,
            new_task_defender_action: false,
            new_task_schedule_type: 0,
            new_task_interval_minutes: 60,
            new_task_daily_hour: 2,
            new_task_daily_minute: 0,
            new_task_weekly_day: 0,
            new_task_weekly_hour: 9,
            new_task_weekly_minute: 0,
            
            // Edit task dialog fields
            edit_task_name: String::new(),
            edit_task_description: String::new(),
            edit_task_type: 0,
            edit_task_ram_threshold: 85,
            edit_task_disk_threshold: 100,
            edit_task_defender_action: false,
            edit_task_schedule_type: 0,
            edit_task_interval_minutes: 60,
            edit_task_daily_hour: 2,
            edit_task_daily_minute: 0,
            edit_task_weekly_day: 0,
            edit_task_weekly_hour: 9,
            edit_task_weekly_minute: 0,
            edit_disk_options: DiskCleaningOptions::default(),
        }
    }

    pub fn update_network_scan(&mut self) {
        if let Some(ref mut limiter) = self.network_limiter {
            match limiter.scan_network_processes() {
                Ok(()) => {
                    tracing::info!("✅ Network scan finished - real-time data");
                }
                Err(e) => {
                    tracing::error!("❌ Network scan error: {}", e);
                }
            }
        }
    }

    pub fn scan_network_processes(&mut self) {
        tracing::info!("🔄 Network scan requested");
        self.update_network_scan();
    }

    pub fn limit_process(&mut self, pid: u32) {
        tracing::info!("🎯 Starting process limitation for PID {}", pid);
        
        if let Some(ref mut limiter) = self.network_limiter {
            // Check if the process exists in the scan
            let process_exists = limiter.get_processes().iter().any(|p| p.pid == pid);
            if !process_exists {
                tracing::warn!("⚠️ Process PID {} not found in network scan", pid);
                return;
            }
            
            let limit_mbps = match crate::network::parse_speed_limit_mbps(&self.speed_limit_input) {
                Ok(mbps) => {
                    tracing::info!("📊 Parsed limit: {} MB/s → OK", mbps);
                    mbps
                },
                Err(e) => {
                    tracing::error!("❌ Invalid limit format '{}': {}", self.speed_limit_input, e);
                    return;
                }
            };
            
            let limit_kbps = (limit_mbps * 1024.0) as u32;
            tracing::info!("🔢 Conversion: {:.1} MB/s → {} KB/s", limit_mbps, limit_kbps);
            
            match limiter.set_process_speed_limit(pid, limit_kbps) {
                Ok(()) => {
                    tracing::info!("✅ QoS limit applied: PID {} → {:.1} MB/s ({} KB/s)", pid, limit_mbps, limit_kbps);
                    
                    // Immediately check if the policy was created
                    match limiter.verify_qos_policies() {
                        Ok(policies) => {
                            let policy_count = policies.len();
                            tracing::info!("📋 Verification: {} QoS policies found after creation", policy_count);
                            
                            // Look for our specific policy
                            let our_policy_name = format!("GameBooster_Limit_{}", pid);
                            let found = policies.iter().any(|p| p.name == our_policy_name);
                            if found {
                                tracing::info!("✅ Policy {} confirmed active", our_policy_name);
                            } else {
                                tracing::warn!("⚠️ Policy {} not found in the active list", our_policy_name);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("⚠️ Unable to verify policies: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("❌ QoS limit failed for PID {}: {}", pid, e);
                }
            }
        } else {
            tracing::error!("❌ NetworkLimiter not initialized for PID {}", pid);
        }
    }

    pub fn remove_process_limit(&mut self, pid: u32) {
        if let Some(ref mut limiter) = self.network_limiter {
            match limiter.remove_process_limit(pid) {
                Ok(()) => {
                    tracing::info!("✅ Limit removed for PID {}", pid);
                }
                Err(e) => {
                    tracing::error!("❌ Failed to remove limit for PID {}: {}", pid, e);
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
            tracing::info!("✅ Limit applied to {} selected processes", self.processes.len());
        }
    }

    pub fn select_all_processes(&mut self) {
        if let Some(ref limiter) = self.network_limiter {
            self.processes.clear();
            for process in limiter.get_processes() {
                self.processes.insert(process.pid);
            }
            tracing::info!("✅ {} processes selected", self.processes.len());
        }
    }

    pub fn deselect_all_processes(&mut self) {
        let count = self.processes.len();
        self.processes.clear();
        tracing::info!("✅ {} processes deselected", count);
    }

    pub fn clear_all_network_limits(&mut self) {
        if let Some(ref mut limiter) = self.network_limiter {
            match limiter.clear_all_limits() {
                Ok(()) => {
                    tracing::info!("✅ All QoS limits cleared");
                }
                Err(e) => {
                    tracing::error!("❌ Failed to clear QoS policies: {}", e);
                }
            }
        }
    }

    // Scheduler management methods
    pub fn check_and_execute_scheduled_tasks(&mut self) {
        if !self.scheduler_running {
            return;
        }

        let now = Local::now();
        // Check every 30 seconds
        if (now - self.last_scheduler_check).num_seconds() < 30 {
            return;
        }

        self.last_scheduler_check = now;
        tracing::info!("🔍 Checking for scheduled tasks to execute...");

        // Get a copy of tasks to avoid borrowing issues
        let tasks_to_check: Vec<_> = self.scheduled_tasks.values().cloned().collect();
        
        for task in tasks_to_check {
            if !task.enabled {
                continue;
            }

            let should_run = self.should_task_run(&task);
            if should_run {
                tracing::info!("⏰ Executing scheduled task: {} ({})", task.name, task.id);
                self.execute_scheduled_task(task);
            }
        }
    }

    fn should_task_run(&self, task: &ScheduledTask) -> bool {
        use crate::scheduler::ScheduleRule;
        
        match &task.schedule {
            ScheduleRule::OnStartup => task.last_run.is_none(),
            ScheduleRule::OnCondition => self.check_task_condition(task),
            ScheduleRule::Interval { minutes } => {
                if let Some(last_run) = task.last_run {
                    let elapsed = Local::now() - last_run;
                    elapsed.num_minutes() >= *minutes as i64
                } else {
                    true // First run
                }
            }
            ScheduleRule::Daily { .. } | ScheduleRule::Weekly { .. } => {
                if let Some(next_run) = task.next_run {
                    Local::now() >= next_run
                } else {
                    false
                }
            }
        }
    }

    fn check_task_condition(&self, task: &ScheduledTask) -> bool {
        use crate::scheduler::TaskType;
        
        match &task.task_type {
            TaskType::CleanRam { threshold_percentage } => {
                let memory_info = crate::memory::get_detailed_system_memory_info();
                let usage_percent = ((memory_info.total_physical - memory_info.avail_physical) * 100) / memory_info.total_physical;
                usage_percent as u8 >= *threshold_percentage
            }
            TaskType::CleanDisk { size_threshold_mb, options } => {
                if let Ok(scan_result) = crate::disk::scan_disk_with_options_sync(options.clone()) {
                    let potential_mb = scan_result.total_space_to_free / 1024 / 1024;
                    potential_mb >= *size_threshold_mb
                } else {
                    false
                }
            }
            TaskType::DefenderToggle { .. } => false, // Manual only
        }
    }

    fn execute_scheduled_task(&mut self, task: ScheduledTask) {
        use crate::scheduler::TaskType;
        
        let task_id = task.id.clone();
        let _task_name = task.name.clone();
        
        match &task.task_type {
            TaskType::CleanRam { threshold_percentage } => {
                tracing::info!("🧹 Executing RAM cleanup task (threshold: {}%)", threshold_percentage);
                match crate::memory::clean_memory() {
                    Ok(results) => {
                        let freed_mb = results.total_freed() / 1024 / 1024;
                        tracing::info!("✅ RAM cleanup completed: {} MB freed", freed_mb);
                        self.mark_task_completed(&task_id, true, None);
                    }
                    Err(e) => {
                        tracing::error!("❌ RAM cleanup failed: {}", e);
                        self.mark_task_completed(&task_id, false, Some(e.to_string()));
                    }
                }
            }
            TaskType::CleanDisk { size_threshold_mb, options } => {
                tracing::info!("🗂️ Executing disk cleanup task (threshold: {} MB)", size_threshold_mb);
                match crate::disk::clean_disk_with_options_sync(options.clone()) {
                    Ok(result) => {
                        let freed_mb = result.total_space_freed / 1024 / 1024;
                        tracing::info!("✅ Disk cleanup completed: {} MB freed", freed_mb);
                        self.mark_task_completed(&task_id, true, None);
                    }
                    Err(e) => {
                        tracing::error!("❌ Disk cleanup failed: {}", e);
                        self.mark_task_completed(&task_id, false, Some(e.to_string()));
                    }
                }
            }
            TaskType::DefenderToggle { enable } => {
                tracing::info!("🛡️ Executing Defender toggle task (enable: {})", enable);
                let result = if *enable {
                    crate::services::defender::DefenderService::enable_immediately()
                } else {
                    crate::services::defender::DefenderService::disable_immediately()
                };
                
                match result {
                    Ok(_) => {
                        tracing::info!("✅ Defender toggle completed successfully");
                        self.mark_task_completed(&task_id, true, None);
                    }
                    Err(e) => {
                        tracing::error!("❌ Defender toggle failed: {}", e);
                        self.mark_task_completed(&task_id, false, Some(e.to_string()));
                    }
                }
            }
        }
    }

    fn mark_task_completed(&mut self, task_id: &str, success: bool, error: Option<String>) {
        // Calculate next run time first
        let next_run = if let Some(task) = self.scheduled_tasks.get(task_id) {
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
        if let Some(task) = self.scheduled_tasks.get_mut(task_id) {
            task.last_run = Some(Local::now());
            task.run_count += 1;
            if success {
                task.success_count += 1;
                task.last_error = None;
            } else {
                task.last_error = error;
            }
            
            task.next_run = next_run;
            
            tracing::info!("📊 Task '{}' completed. Stats: {}/{} successes", 
                          task.name, task.success_count, task.run_count);
            
            // Update the task in the scheduler and save
            self.task_scheduler.update_task(task.clone());
            if let Err(e) = self.task_scheduler.save_tasks() {
                tracing::error!("❌ Failed to save task completion status: {}", e);
            }
        }
    }

    fn calculate_next_run(&self, task: &ScheduledTask) -> Option<DateTime<Local>> {
        use crate::scheduler::ScheduleRule;
        use chrono::{Duration, Datelike, Weekday as ChronoWeekday};
        
        match &task.schedule {
            ScheduleRule::OnStartup | ScheduleRule::OnCondition => None,
            ScheduleRule::Interval { minutes } => {
                Some(Local::now() + Duration::minutes(*minutes as i64))
            }
            ScheduleRule::Daily { time } => {
                let now = Local::now();
                let today = now.date_naive();
                let target_datetime = today.and_time(*time);
                
                if let Some(target_local) = Local.from_local_datetime(&target_datetime).earliest() {
                    if target_local > now {
                        Some(target_local)
                    } else {
                        // Schedule for tomorrow
                        let tomorrow = today + Duration::days(1);
                        let tomorrow_target = tomorrow.and_time(*time);
                        Local.from_local_datetime(&tomorrow_target).earliest()
                    }
                } else {
                    None
                }
            }
            ScheduleRule::Weekly { weekday, time } => {
                let now = Local::now();
                let today = now.date_naive();
                let current_weekday = today.weekday();
                
                let target_weekday = match weekday {
                    chrono::Weekday::Mon => ChronoWeekday::Mon,
                    chrono::Weekday::Tue => ChronoWeekday::Tue,
                    chrono::Weekday::Wed => ChronoWeekday::Wed,
                    chrono::Weekday::Thu => ChronoWeekday::Thu,
                    chrono::Weekday::Fri => ChronoWeekday::Fri,
                    chrono::Weekday::Sat => ChronoWeekday::Sat,
                    chrono::Weekday::Sun => ChronoWeekday::Sun,
                };
                
                let days_until_target = if target_weekday == current_weekday {
                    let target_datetime = today.and_time(*time);
                    if let Some(target_local) = Local.from_local_datetime(&target_datetime).earliest() {
                        if target_local > now {
                            0
                        } else {
                            7
                        }
                    } else {
                        7
                    }
                } else {
                    let target_num = target_weekday.num_days_from_monday();
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

    // Task management methods with persistence
    pub fn add_scheduled_task(&mut self, task: ScheduledTask) {
        let task_id = task.id.clone();
        let task_name = task.name.clone();
        
        // Add to both app and scheduler
        self.scheduled_tasks.insert(task_id.clone(), task.clone());
        self.task_scheduler.add_task(task);
        
        // Save to file
        if let Err(e) = self.task_scheduler.save_tasks() {
            tracing::error!("❌ Failed to save tasks after adding '{}': {}", task_name, e);
        } else {
            tracing::info!("✅ Task '{}' added and saved successfully", task_name);
        }
    }

    pub fn remove_scheduled_task(&mut self, task_id: &str) {
        if let Some(task) = self.scheduled_tasks.remove(task_id) {
            self.task_scheduler.remove_task(task_id);
            
            // Save to file
            if let Err(e) = self.task_scheduler.save_tasks() {
                tracing::error!("❌ Failed to save tasks after removing '{}': {}", task.name, e);
            } else {
                tracing::info!("✅ Task '{}' removed and saved successfully", task.name);
            }
        }
    }

    pub fn update_scheduled_task(&mut self, task: ScheduledTask) {
        let task_id = task.id.clone();
        let task_name = task.name.clone();
        
        // Update in both app and scheduler
        self.scheduled_tasks.insert(task_id.clone(), task.clone());
        self.task_scheduler.update_task(task);
        
        // Save to file
        if let Err(e) = self.task_scheduler.save_tasks() {
            tracing::error!("❌ Failed to save tasks after updating '{}': {}", task_name, e);
        } else {
            tracing::info!("✅ Task '{}' updated and saved successfully", task_name);
        }
    }
}

impl eframe::App for CleanRamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(self.theme.visuals.clone());

        // Check and execute scheduled tasks if scheduler is running
        self.check_and_execute_scheduled_tasks();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.active_tab == Tab::Memory, "🧠 Memory").clicked() {
                    self.active_tab = Tab::Memory;
                }
                if ui.selectable_label(self.active_tab == Tab::Hdd, "💾 Disk").clicked() {
                    self.active_tab = Tab::Hdd;
                }
                if ui.selectable_label(self.active_tab == Tab::Services, "🛡️ Services").clicked() {
                    self.active_tab = Tab::Services;
                }
                if ui.selectable_label(self.active_tab == Tab::Scheduler, "⏰ Scheduler").clicked() {
                    self.active_tab = Tab::Scheduler;
                }
                if ui.selectable_label(self.active_tab == Tab::Network, "🌐 Network").clicked() {
                    self.active_tab = Tab::Network;
                }
                if ui.selectable_label(self.active_tab == Tab::Settings, "⚙️ Settings").clicked() {
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
            // No automatic check on startup to avoid opening PowerShell
        }
    }
}