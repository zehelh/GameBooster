// Windows Defender management

use anyhow::Result;
use std::process::Command;
use chrono::{DateTime, Local, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenderStatus {
    pub real_time_protection: bool,
    pub cloud_protection: bool,
    pub automatic_sample_submission: bool,
    pub tamper_protection: bool,
    pub last_check: DateTime<Local>,
}

impl DefenderStatus {
    pub fn check_current_status() -> Result<Self> {
        // Use PowerShell to check Windows Defender status
        let output = Command::new("powershell")
            .args(&["-Command", "Get-MpPreference | Select-Object DisableRealtimeMonitoring, MAPSReporting, SubmitSamplesConsent"])
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse the output to determine status
        let real_time_disabled = output_str.contains("DisableRealtimeMonitoring : True");
        let cloud_disabled = output_str.contains("MAPSReporting : 0");
        let samples_disabled = output_str.contains("SubmitSamplesConsent : 0");
        
        Ok(Self {
            real_time_protection: !real_time_disabled,
            cloud_protection: !cloud_disabled,
            automatic_sample_submission: !samples_disabled,
            tamper_protection: false, // Default to false, would need additional check
            last_check: Local::now(),
        })
    }
}

pub async fn disable_defender_temporarily() -> Result<bool> {
    // This is a very sensitive operation that should only be done with explicit user consent
    // and clear warnings about security implications
    
    // Check if running as administrator
    if !is_elevated::is_elevated() {
        return Err(anyhow::anyhow!("Administrator privileges required to modify Windows Defender"));
    }

    // Disable real-time protection temporarily
    let result = Command::new("powershell")
        .args(&["-Command", "Set-MpPreference -DisableRealtimeMonitoring $true"])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(true)
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Failed to disable Defender: {}", error))
            }
        }
        Err(e) => Err(anyhow::anyhow!("Command execution failed: {}", e)),
    }
}

pub async fn enable_defender() -> Result<bool> {
    // Re-enable Windows Defender
    if !is_elevated::is_elevated() {
        return Err(anyhow::anyhow!("Administrator privileges required to modify Windows Defender"));
    }

    let result = Command::new("powershell")
        .args(&["-Command", "Set-MpPreference -DisableRealtimeMonitoring $false"])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(true)
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Failed to enable Defender: {}", error))
            }
        }
        Err(e) => Err(anyhow::anyhow!("Command execution failed: {}", e)),
    }
}

pub async fn schedule_defender_reactivation(hours: u32) -> Result<()> {
    // Schedule automatic re-activation of Windows Defender after specified hours
    // This could be implemented using Windows Task Scheduler
    
    let command = format!(
        "schtasks /create /tn \"GameBooster_DefenderReactivation\" /tr \"powershell.exe -Command 'Set-MpPreference -DisableRealtimeMonitoring $false'\" /sc once /st {}",
        (Local::now() + Duration::hours(hours as i64)).format("%H:%M")
    );

    let result = Command::new("cmd")
        .args(&["/C", &command])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Failed to schedule reactivation: {}", error))
            }
        }
        Err(e) => Err(anyhow::anyhow!("Command execution failed: {}", e)),
    }
}

pub fn get_defender_exclusions() -> Result<Vec<String>> {
    // Get current exclusions list
    let output = Command::new("powershell")
        .args(&["-Command", "Get-MpPreference | Select-Object -ExpandProperty ExclusionPath"])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let exclusions: Vec<String> = output_str
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    Ok(exclusions)
}

pub fn add_exclusion_path(path: &str) -> Result<()> {
    // Add a path to Windows Defender exclusions
    if !is_elevated::is_elevated() {
        return Err(anyhow::anyhow!("Administrator privileges required"));
    }

    let command = format!("Add-MpPreference -ExclusionPath '{}'", path);
    
    let result = Command::new("powershell")
        .args(&["-Command", &command])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Failed to add exclusion: {}", error))
            }
        }
        Err(e) => Err(anyhow::anyhow!("Command execution failed: {}", e)),
    }
}
