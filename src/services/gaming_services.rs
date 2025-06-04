// Gaming-related Windows services optimization

use anyhow::Result;
use std::process::Command;
use crate::services::{ServiceOperation, ServiceAction};
use chrono::Local;

pub async fn optimize_gaming_services() -> Result<Vec<ServiceOperation>> {
    let mut operations = Vec::new();

    // List of services that can be safely disabled/stopped for gaming
    let gaming_optimizations = vec![
        ("Windows Search", "WSearch", ServiceAction::Stop),
        ("Windows Update", "wuauserv", ServiceAction::Stop),
        ("Superfetch", "SysMain", ServiceAction::Stop),
        ("Print Spooler", "Spooler", ServiceAction::Stop),
        ("Fax", "Fax", ServiceAction::Stop),
        ("Tablet PC Input Service", "TabletInputService", ServiceAction::Stop),
        ("Windows Error Reporting", "WerSvc", ServiceAction::Stop),
    ];

    for (display_name, service_name, action) in gaming_optimizations {
        let operation = perform_service_operation(display_name, service_name, action).await;
        operations.push(operation);
    }

    Ok(operations)
}

async fn perform_service_operation(
    display_name: &str,
    service_name: &str,
    action: ServiceAction,
) -> ServiceOperation {
    let command_args = match action {
        ServiceAction::Stop => vec!["stop", service_name],
        ServiceAction::Start => vec!["start", service_name],
        ServiceAction::Disable => vec!["config", service_name, "start=", "disabled"],
        ServiceAction::Enable => vec!["config", service_name, "start=", "auto"],
    };

    let result = Command::new("sc")
        .args(&command_args)
        .output();

    match result {
        Ok(output) => {
            let success = output.status.success();
            let error_message = if !success {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            } else {
                None
            };

            ServiceOperation {
                service_name: service_name.to_string(),
                display_name: display_name.to_string(),
                action,
                timestamp: Local::now(),
                success,
                error_message,
            }
        }
        Err(e) => ServiceOperation {
            service_name: service_name.to_string(),
            display_name: display_name.to_string(),
            action,
            timestamp: Local::now(),
            success: false,
            error_message: Some(e.to_string()),
        },
    }
}

pub async fn restore_services() -> Result<Vec<ServiceOperation>> {
    let mut operations = Vec::new();

    // Restore services to their default state
    let service_restorations = vec![
        ("Windows Search", "WSearch", ServiceAction::Start),
        ("Windows Update", "wuauserv", ServiceAction::Start),
        ("Superfetch", "SysMain", ServiceAction::Start),
        ("Print Spooler", "Spooler", ServiceAction::Start),
        ("Windows Error Reporting", "WerSvc", ServiceAction::Start),
    ];

    for (display_name, service_name, action) in service_restorations {
        let operation = perform_service_operation(display_name, service_name, action).await;
        operations.push(operation);
    }

    Ok(operations)
}

pub fn get_service_recommendations() -> Vec<(String, String, String)> {
    // Returns (service_name, display_name, description) for services that can be optimized
    vec![
        (
            "WSearch".to_string(),
            "Windows Search".to_string(),
            "Indexes files for faster searching. Can be disabled for gaming.".to_string(),
        ),
        (
            "SysMain".to_string(),
            "Superfetch".to_string(),
            "Preloads frequently used apps. May cause disk usage during gaming.".to_string(),
        ),
        (
            "wuauserv".to_string(),
            "Windows Update".to_string(),
            "Handles Windows updates. Can be temporarily stopped.".to_string(),
        ),
        (
            "Spooler".to_string(),
            "Print Spooler".to_string(),
            "Manages printing. Safe to disable if no printer is used.".to_string(),
        ),
        (
            "WerSvc".to_string(),
            "Windows Error Reporting".to_string(),
            "Collects error reports. Can be disabled for privacy and performance.".to_string(),
        ),
    ]
}

pub fn is_service_safe_to_modify(service_name: &str) -> bool {
    // List of services that are safe to modify without breaking the system
    let safe_services = vec![
        "WSearch", "SysMain", "wuauserv", "Spooler", "Fax",
        "TabletInputService", "WerSvc", "Themes", "Browser",
    ];

    safe_services.contains(&service_name)
}
