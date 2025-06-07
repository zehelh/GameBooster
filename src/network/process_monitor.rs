use std::collections::HashMap;
use std::time::{Instant};
use sysinfo::{System, Pid};
use anyhow::Result;

// A struct internal to this module to hold process information.
#[derive(Debug, Clone)]
pub struct ProcessNetworkInfo {
    pub pid: u32,
    pub name: String,
}

pub struct ProcessMonitor {
    system: System,
    processes: HashMap<Pid, ProcessNetworkInfo>,
    last_refresh: Instant,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            processes: HashMap::new(),
            last_refresh: Instant::now(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_processes();
    }

    pub fn get_processes_with_network(&self) -> Vec<ProcessNetworkInfo> {
        let mut result = Vec::new();
        for (pid, process) in self.system.processes() {
            // A basic heuristic: if a process has a name, consider it.
            // The is_windows_system_process filter can be applied later in the UI logic if needed.
            result.push(ProcessNetworkInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
            });
        }
        result
    }
}

// This function is the main entry point for this module
pub async fn scan_network_processes() -> Result<Vec<ProcessNetworkInfo>> {
    let mut monitor = ProcessMonitor::new();
    monitor.refresh();
    let processes = monitor.get_processes_with_network();
    Ok(processes)
}
