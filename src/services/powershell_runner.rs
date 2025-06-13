// PowerShell command runner with hidden windows
// Manages PowerShell commands execution in background without visible windows

use anyhow::Result;
use thiserror::Error;

#[cfg(target_os = "windows")]
use async_process::Command;

// Windows constant to hide the window
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Error)]
pub enum PowerShellExecutionError {
    #[error("La commande PowerShell a échoué avec le code de sortie {0}. Erreur : {1}")]
    CommandFailed(i32, String),
    #[error("Erreur d'entrée/sortie lors de l'exécution de la commande : {0}")]
    IoError(#[from] std::io::Error),
    #[error("Fonctionnalité non disponible sur cette plateforme")]
    NotAvailable,
}

/// Exécute une commande PowerShell de manière asynchrone et cachée.
#[cfg(target_os = "windows")]
pub async fn run_powershell_command(command: &str) -> Result<String, PowerShellExecutionError> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            command,
        ])
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(PowerShellExecutionError::CommandFailed(
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

#[cfg(not(target_os = "windows"))]
pub async fn run_powershell_command(command: &str) -> Result<String, PowerShellExecutionError> {
    let _ = command; // Mark as used
    Err(PowerShellExecutionError::NotAvailable)
}
