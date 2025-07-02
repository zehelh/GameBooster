// Windows Defender management

use anyhow::Result;
use crate::services::winapi_defender::DefenderManager;

#[derive(Debug, Clone, Default)]
pub struct DefenderStatus {
    pub real_time_protection: bool,
    pub cloud_protection: bool,
    pub automatic_sample_submission: bool,
    pub tamper_protection: bool,
    pub status_text: String,
    pub last_operation_results: Vec<String>,
}

pub struct DefenderService;

impl DefenderService {
    /// Check current Defender status with detailed information
    pub fn get_status() -> Result<DefenderStatus> {
        match DefenderManager::check_defender_status() {
            Ok(status) => {
                let mut defender_status = DefenderStatus {
                    real_time_protection: status.real_time_protection,
                    cloud_protection: status.cloud_protection,
                    automatic_sample_submission: status.automatic_sample_submission,
                    tamper_protection: status.tamper_protection,
                    status_text: if status.real_time_protection {
                        "🛡️ Actif - Protection en temps réel".to_string()
                    } else {
                        "❌ Désactivé - Protection arrêtée".to_string()
                    },
                    last_operation_results: Vec::new(),
                };

                // Add detailed status info
                let mut details = Vec::new();
                if status.real_time_protection {
                    details.push("Protection temps réel: ✅ ACTIVE".to_string());
                } else {
                    details.push("Protection temps réel: ❌ INACTIVE".to_string());
                }

                if status.tamper_protection {
                    details.push("Protection falsification: 🔒 VERROUILLÉE".to_string());
                } else {
                    details.push("Protection falsification: 🔓 DÉVERROUILLÉE".to_string());
                }

                if status.cloud_protection {
                    details.push("Protection cloud: ☁️ ACTIVE".to_string());
                } else {
                    details.push("Protection cloud: ❌ INACTIVE".to_string());
                }

                defender_status.last_operation_results = details;
                Ok(defender_status)
            }
            Err(e) => {
                Ok(DefenderStatus {
                    status_text: format!("❓ Statut inconnu: {}", e),
                    last_operation_results: vec![format!("Erreur de vérification: {}", e)],
                    ..Default::default()
                })
            }
        }
    }    /// Disable Defender immediately with detailed feedback
    pub fn disable_immediately() -> Result<DefenderStatus> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(Self::disable_immediately_async())
    }

    /// Async version for internal use
    async fn disable_immediately_async() -> Result<DefenderStatus> {
        let results = DefenderManager::disable_defender_immediately().await?;
        
        // Wait a moment for changes to take effect
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
        
        let mut status = Self::get_status().unwrap_or_default();
        status.last_operation_results = results;
        
        // Update status text based on results
        if !status.real_time_protection {
            status.status_text = "🎉 DISABLED - All protections stopped".to_string();
        } else {
            status.status_text = "⚠️ PARTIALLY DISABLED - Check results".to_string();
        }
        
        Ok(status)
    }

    /// Enable Defender immediately with detailed feedback
    pub fn enable_immediately() -> Result<DefenderStatus> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(Self::enable_immediately_async())
    }

    /// Async version for internal use
    async fn enable_immediately_async() -> Result<DefenderStatus> {
        let results = DefenderManager::enable_defender_immediately().await?;
        
        // Wait a moment for changes to take effect
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
        
        let mut status = Self::get_status().unwrap_or_default();
        status.last_operation_results = results;
        
        // Update status text based on results
        if status.real_time_protection {
            status.status_text = "🛡️ RÉACTIVÉ - Protection restaurée".to_string();
        } else {
            status.status_text = "⚠️ RÉACTIVATION PARTIELLE - Redémarrage possible requis".to_string();
        }
        
        Ok(status)
    }

    /// Quick status check (lighter than full get_status)
    pub fn is_active() -> bool {
        Self::get_status()
            .map(|s| s.real_time_protection)
            .unwrap_or(true) // Default to active if can't check
    }
}

/*
// Ces fonctions utilisant Command sont commentées car elles ne sont pas utilisées
// et on veut éviter les outils externes

pub async fn schedule_defender_reactivation(hours: u32) -> Result<()> {
    // TODO: Implémenter via l'API Windows Task Scheduler au lieu de Command
    Err(anyhow!("Scheduler functionality not yet implemented via Registry API"))
}

pub fn get_defender_exclusions() -> Result<Vec<String>> {
    // TODO: Implémenter via l'API Registry au lieu de PowerShell
    Err(anyhow!("Exclusions reading not yet implemented via Registry API"))
}

pub fn add_exclusion_path(path: &str) -> Result<()> {
    // TODO: Implémenter via l'API Registry au lieu de PowerShell
    Err(anyhow!("Exclusions management not yet implemented via Registry API"))
}
*/ 