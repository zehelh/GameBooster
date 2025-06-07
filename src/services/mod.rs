// Windows services optimization module
pub mod defender;
pub mod powershell_runner;
pub mod winapi_defender;
pub mod winapi_service_manager;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;
use chrono::{DateTime, Local};
use crate::services::defender::DefenderService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceAction {
    Disable,
    Enable,
    Stop,
    Start,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOperation {
    pub service_name: String,
    pub display_name: String,
    pub action: ServiceAction,
    pub timestamp: DateTime<Local>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesOptimizationResults {
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub operations: Vec<ServiceOperation>,
    pub defender_disabled: bool,
    pub services_optimized: u32,
    pub errors: Vec<String>,
    pub is_completed: bool,
}

impl ServicesOptimizationResults {
    pub fn new() -> Self {
        Self {
            start_time: Local::now(),
            end_time: None,
            operations: Vec::new(),
            defender_disabled: false,
            services_optimized: 0,
            errors: Vec::new(),
            is_completed: false,
        }
    }

    pub fn add_operation(&mut self, operation: ServiceOperation) {
        if operation.success {
            self.services_optimized += 1;
        } else if let Some(error) = &operation.error_message {
            self.errors.push(error.clone());
        }
        self.operations.push(operation);
    }

    pub fn complete(&mut self) {
        self.end_time = Some(Local::now());
        self.is_completed = true;
    }
}

pub async fn optimize_services_for_gaming() -> Result<ServicesOptimizationResults> {
    let mut results = ServicesOptimizationResults::new();

    // Disable Windows Defender (with user consent)
    match handle_disable_defender().await {
        Ok(disabled) => {
            results.defender_disabled = disabled;
            if disabled {
                results.add_operation(ServiceOperation {
                    service_name: "Windows Defender".to_string(),
                    display_name: "Windows Defender Antivirus Service".to_string(),
                    action: ServiceAction::Disable,
                    timestamp: Local::now(),
                    success: true,
                    error_message: None,
                });
            }
        }
        Err(e) => {
            results.add_operation(ServiceOperation {
                service_name: "Windows Defender".to_string(),
                display_name: "Windows Defender Antivirus Service".to_string(),
                action: ServiceAction::Disable,
                timestamp: Local::now(),
                success: false,
                error_message: Some(e),
            });
        }
    }

    results.complete();
    Ok(results)
}

pub fn is_service_running(service_name: &str) -> Result<bool> {
    let output = Command::new("sc")
        .args(&["query", service_name])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.contains("RUNNING"))
}

pub fn get_service_status(service_name: &str) -> Result<String> {
    let output = Command::new("sc")
        .args(&["query", service_name])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    
    if output_str.contains("RUNNING") {
        Ok("Running".to_string())
    } else if output_str.contains("STOPPED") {
        Ok("Stopped".to_string())
    } else if output_str.contains("START_PENDING") {
        Ok("Starting".to_string())
    } else if output_str.contains("STOP_PENDING") {
        Ok("Stopping".to_string())
    } else {
        Ok("Unknown".to_string())
    }
}

pub async fn handle_disable_defender() -> Result<bool, String> {
    match DefenderService::disable_immediately() {
        Ok(status) => {
            // Check if the operation was successful based on the status
            Ok(!status.real_time_protection)
        }
        Err(e) => Err(format!("Failed to disable Windows Defender: {}", e)),
    }
}
