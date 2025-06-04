// Configuration management for scheduler

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::scheduler::ScheduledTask;

#[derive(Debug, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub tasks: Vec<ScheduledTask>,
    pub auto_start: bool,
    pub log_activities: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            auto_start: false,
            log_activities: true,
        }
    }
}

impl SchedulerConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.as_ref().exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let config: SchedulerConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add_task(&mut self, task: ScheduledTask) {
        // Remove existing task with same ID if exists
        self.tasks.retain(|t| t.id != task.id);
        self.tasks.push(task);
    }

    pub fn remove_task(&mut self, task_id: &str) {
        self.tasks.retain(|t| t.id != task_id);
    }

    pub fn get_task(&self, task_id: &str) -> Option<&ScheduledTask> {
        self.tasks.iter().find(|t| t.id == task_id)
    }

    pub fn get_task_mut(&mut self, task_id: &str) -> Option<&mut ScheduledTask> {
        self.tasks.iter_mut().find(|t| t.id == task_id)
    }
}
