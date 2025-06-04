// Network bandwidth limiting module
pub mod process_monitor;
pub mod bandwidth_control;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkProcess {
    pub pid: u32,
    pub name: String,
    pub executable_path: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections: u32,
    pub is_windows_process: bool,
    pub bandwidth_limit: Option<u64>, // bytes per second
    pub is_blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLimitRule {
    pub process_name: String,
    pub limit_download: Option<u64>, // bytes per second
    pub limit_upload: Option<u64>,   // bytes per second
    pub is_blocked: bool,
    pub created_at: DateTime<Local>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLimitingResults {
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub rules_applied: u32,
    pub processes_limited: u32,
    pub processes_blocked: u32,
    pub errors: Vec<String>,
    pub is_completed: bool,
}

impl NetworkLimitingResults {
    pub fn new() -> Self {
        Self {
            start_time: Local::now(),
            end_time: None,
            rules_applied: 0,
            processes_limited: 0,
            processes_blocked: 0,
            errors: Vec::new(),
            is_completed: false,
        }
    }

    pub fn complete(&mut self) {
        self.end_time = Some(Local::now());
        self.is_completed = true;
    }
}

pub struct NetworkLimiter {
    active_rules: HashMap<String, NetworkLimitRule>,
    monitored_processes: HashMap<u32, NetworkProcess>,
}

impl NetworkLimiter {
    pub fn new() -> Self {
        Self {
            active_rules: HashMap::new(),
            monitored_processes: HashMap::new(),
        }
    }

    pub async fn scan_network_processes(&mut self) -> Result<Vec<NetworkProcess>> {
        process_monitor::scan_network_processes().await
    }

    pub fn add_bandwidth_rule(&mut self, rule: NetworkLimitRule) {
        self.active_rules.insert(rule.process_name.clone(), rule);
    }

    pub fn remove_bandwidth_rule(&mut self, process_name: &str) {
        self.active_rules.remove(process_name);
    }

    pub async fn apply_network_limits(&mut self) -> Result<NetworkLimitingResults> {
        let mut results = NetworkLimitingResults::new();

        // Get current network processes
        let processes = self.scan_network_processes().await?;
        
        for process in processes {
            if let Some(rule) = self.active_rules.get(&process.name) {
                if rule.enabled {
                    if rule.is_blocked {
                        // Block the process completely
                        match bandwidth_control::block_process_network(&process.name).await {
                            Ok(_) => {
                                results.processes_blocked += 1;
                                results.rules_applied += 1;
                            }
                            Err(e) => {
                                results.errors.push(format!("Failed to block {}: {}", process.name, e));
                            }
                        }
                    } else {
                        // Apply bandwidth limits
                        match bandwidth_control::limit_process_bandwidth(
                            &process.name,
                            rule.limit_download,
                            rule.limit_upload,
                        ).await {
                            Ok(_) => {
                                results.processes_limited += 1;
                                results.rules_applied += 1;
                            }
                            Err(e) => {
                                results.errors.push(format!("Failed to limit {}: {}", process.name, e));
                            }
                        }
                    }
                }
            }
        }

        results.complete();
        Ok(results)
    }

    pub fn get_active_rules(&self) -> &HashMap<String, NetworkLimitRule> {
        &self.active_rules
    }

    pub fn clear_all_rules(&mut self) {
        self.active_rules.clear();
    }
}

pub async fn get_network_usage_by_process() -> Result<Vec<NetworkProcess>> {
    process_monitor::scan_network_processes().await
}

pub fn is_windows_system_process(process_name: &str) -> bool {
    let windows_processes = vec![
        "svchost.exe", "System", "Registry", "dwm.exe", "winlogon.exe",
        "csrss.exe", "lsass.exe", "services.exe", "spoolsv.exe",
        "explorer.exe", "taskhost.exe", "rundll32.exe", "dllhost.exe",
        "msiexec.exe", "conhost.exe", "audiodg.exe", "wininit.exe",
    ];

    windows_processes.iter().any(|&sys_proc| 
        process_name.to_lowercase().contains(&sys_proc.to_lowercase())
    )
}
