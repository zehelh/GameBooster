// Scheduler module for automatic cleaning tasks
pub mod task;
pub mod config;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    CleanRam,
    CleanDisk,
    OptimizeServices,
    NetworkLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleRule {
    OnStartup,
    Hourly(u32), // Every X hours
    Daily(u32),  // At specific hour (0-23)
    Weekly(u32, u32), // Day of week (0-6), hour (0-23)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub task_type: TaskType,
    pub schedule: ScheduleRule,
    pub enabled: bool,
    pub last_run: Option<DateTime<Local>>,
    pub next_run: Option<DateTime<Local>>,
}

pub struct TaskScheduler {
    tasks: HashMap<String, ScheduledTask>,
    config_path: String,
}

impl TaskScheduler {
    pub fn new(config_path: &str) -> Self {
        Self {
            tasks: HashMap::new(),
            config_path: config_path.to_string(),
        }
    }

    pub fn load_tasks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load tasks from config file
        Ok(())
    }

    pub fn save_tasks(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Save tasks to config file
        Ok(())
    }

    pub fn add_task(&mut self, task: ScheduledTask) {
        self.tasks.insert(task.id.clone(), task);
    }

    pub fn get_pending_tasks(&self) -> Vec<&ScheduledTask> {
        // Return tasks that need to be executed
        Vec::new()
    }

    pub fn calculate_next_run(&self, task: &ScheduledTask) -> Option<DateTime<Local>> {
        // Calculate next execution time based on schedule rule
        None
    }
}
