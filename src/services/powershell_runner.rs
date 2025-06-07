// PowerShell command runner with hidden windows
// Manages PowerShell commands execution in background without visible windows

use anyhow::Result;
use async_process::Command;
use thiserror::Error;

// Windows constant to hide the window
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Error)]
pub enum PowerShellExecutionError {
    #[error("La commande PowerShell a échoué avec le code de sortie {0}. Erreur : {1}")]
    CommandFailed(i32, String),
    #[error("Erreur d'entrée/sortie lors de l'exécution de la commande : {0}")]
    IoError(#[from] std::io::Error),
}

/// Exécute une commande PowerShell de manière asynchrone et cachée.
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
