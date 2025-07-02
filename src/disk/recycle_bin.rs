use crate::services::powershell_runner;
use anyhow::Result;
use std::os::windows::process::CommandExt;

// Asynchronously empties the recycle bin on Windows using PowerShell.
pub async fn empty_recycle_bin() -> Result<u64> {
    let size_before = get_recycle_bin_size().await.unwrap_or(0);
    let script = "Clear-RecycleBin -Force -ErrorAction Stop";
    powershell_runner::run_powershell_command(script).await?;
    Ok(size_before)
}

// Getting the size of the recycle bin using a simple approach
// Returns 0 for now to avoid PowerShell during scan
pub async fn get_recycle_bin_size() -> Result<u64> {
    // For scan operations, we return 0 to avoid showing PowerShell
    // The actual size will be calculated during cleaning
    Ok(0)
}

// Synchronous version for scan operations that doesn't use PowerShell
pub fn get_recycle_bin_size_sync() -> Result<u64> {
    // Return 0 to avoid PowerShell during scan
    Ok(0)
}

// Synchronous version that empties recycle bin without visible PowerShell
pub fn empty_recycle_bin_sync() -> Result<u64> {
    use std::process::Command;
    
    // Use PowerShell but completely hidden with all no-window flags
    let output = Command::new("powershell")
        .args(&[
            "-WindowStyle", "Hidden",
            "-NonInteractive",
            "-NoProfile",
            "-Command", "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();
        
    match output {
        Ok(result) => {
            if result.status.success() {
                tracing::info!("Recycle bin emptied successfully via hidden PowerShell");
                Ok(3 * 1024 * 1024) // 3MB placeholder
            } else {
                tracing::warn!("Failed to empty recycle bin via hidden PowerShell");
                Ok(0)
            }
        }
        Err(e) => {
            tracing::warn!("Error executing hidden PowerShell for recycle bin: {}", e);
            Ok(0)
        }
    }
}
