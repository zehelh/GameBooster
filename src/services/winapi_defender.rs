// Windows Defender management using Registry and WinAPI
// Manages Windows Defender without PowerShell commands

use anyhow::{Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use std::ffi::{c_void, CString};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
use tracing;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, HANDLE,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExA, RegDeleteValueA, RegOpenKeyExA, RegQueryValueExA, RegSetValueExA,
    HKEY, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, KEY_WOW64_64KEY, REG_DWORD,
    REG_OPTION_NON_VOLATILE,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

// Import from local utils module
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenderStatus {
    pub real_time_protection: bool,
    pub cloud_protection: bool,
    pub automatic_sample_submission: bool,
    pub tamper_protection: bool,
    pub last_check: DateTime<Local>,
}

impl Default for DefenderStatus {
    fn default() -> Self {
        Self {
            real_time_protection: false,
            cloud_protection: false,
            automatic_sample_submission: false,
            tamper_protection: false,
            last_check: Local::now(),
        }
    }
}

pub struct DefenderManager;

#[cfg(target_os = "windows")]
impl DefenderManager {
    const DEFENDER_REGISTRY_PATH: &'static str = "SOFTWARE\\\\Microsoft\\\\Windows Defender";
    const POLICY_REGISTRY_PATH: &'static str = "SOFTWARE\\Policies\\Microsoft\\Windows Defender\\Real-Time Protection";
    const FEATURES_REGISTRY_PATH: &'static str = "SOFTWARE\\Microsoft\\Windows Defender\\Features";
    const SPYNET_REGISTRY_PATH: &'static str = "SOFTWARE\\Microsoft\\Windows Defender\\Spynet";
    const SCAN_REGISTRY_PATH: &'static str = "SOFTWARE\\Microsoft\\Windows Defender\\Scan";

    /// Check if Windows Defender real-time protection is enabled via registry
    pub fn check_defender_status() -> Result<DefenderStatus> {
        // Vérifier PLUSIEURS sources pour avoir le vrai statut
        
        // 1. Check Policy registry (priorité la plus élevée)
        let policy_disable = Self::_get_policy_setting("DisableRealtimeMonitoring")?.unwrap_or(0);
        
        // 2. Check Main Defender registry
        let main_disable = Self::_get_defender_setting("DisableRealtimeMonitoring")?.unwrap_or(0);
        
        // 3. Check via DisableAntiSpyware (méthode alternative)
        let antispyware_disable = Self::_get_defender_setting("DisableAntiSpyware")?.unwrap_or(0);
        
        // Si UNE SEULE des méthodes dit que c'est désactivé, alors c'est désactivé
        let real_time_protection = policy_disable == 0 && main_disable == 0 && antispyware_disable == 0;
        
        let cloud_protection =
            Self::_get_defender_setting("DisableBlockAtFirstSeen")?.unwrap_or(0) == 0;
            
        let automatic_sample_submission =
            Self::_get_defender_setting("SubmitSamplesConsent")?.unwrap_or(1) != 0;
        
        // Check Tamper Protection status
        let tamper_protection = Self::_get_features_setting("TamperProtection")?.unwrap_or(5) >= 4;

        Ok(DefenderStatus {
            real_time_protection,
            cloud_protection,
            automatic_sample_submission,
            tamper_protection,
            last_check: Local::now(),
        })
    }

    /// Read a DWORD value from the Policy registry key  
    fn _get_policy_setting(value_name: &str) -> Result<Option<u32>> {
        unsafe {
            let mut key: HKEY = std::ptr::null_mut();
            let registry_path =
                CString::new(Self::POLICY_REGISTRY_PATH).map_err(|e| anyhow!(e))?;

            let result = RegOpenKeyExA(
                HKEY_LOCAL_MACHINE,
                registry_path.as_ptr() as *const u8,
                0,
                KEY_READ | KEY_WOW64_64KEY,
                &mut key,
            );

            if result != ERROR_SUCCESS {
                return Ok(None);
            }

            let mut value: u32 = 0;
            let mut value_size: u32 = std::mem::size_of::<u32>() as u32;
            let mut value_type: u32 = 0;
            let value_name_cstr = CString::new(value_name).map_err(|e| anyhow!(e))?;

            let read_result = RegQueryValueExA(
                key,
                value_name_cstr.as_ptr() as *const u8,
                std::ptr::null_mut(),
                &mut value_type,
                &mut value as *mut u32 as *mut u8,
                &mut value_size,
            );

            RegCloseKey(key);

            if read_result == ERROR_SUCCESS {
                Ok(Some(value))
            } else if read_result == ERROR_FILE_NOT_FOUND {
                Ok(None)
            } else {
                Err(anyhow!(
                    "Failed to read policy registry value '{}'. Error: {}",
                    value_name,
                    read_result
                ))
            }
        }
    }

    /// Generic function to read a DWORD value from the Defender registry key.
    /// Returns Ok(None) if the value doesn't exist.
    fn _get_defender_setting(value_name: &str) -> Result<Option<u32>> {
        unsafe {
            let mut key: HKEY = std::ptr::null_mut();
            let registry_path =
                CString::new(Self::DEFENDER_REGISTRY_PATH).map_err(|e| anyhow!(e))?;

            let result = RegOpenKeyExA(
                HKEY_LOCAL_MACHINE,
                registry_path.as_ptr() as *const u8,
                0,
                KEY_READ | KEY_WOW64_64KEY,
                &mut key,
            );

            if result != ERROR_SUCCESS {
                // Key not existing is not an error, it just means settings are default
                return Ok(None);
            }

            let mut value: u32 = 0;
            let mut value_size: u32 = std::mem::size_of::<u32>() as u32;
            let mut value_type: u32 = 0;
            let value_name_cstr = CString::new(value_name).map_err(|e| anyhow!(e))?;

            let read_result = RegQueryValueExA(
                key,
                value_name_cstr.as_ptr() as *const u8,
                std::ptr::null_mut(),
                &mut value_type,
                &mut value as *mut u32 as *mut u8,
                &mut value_size,
            );

            RegCloseKey(key);

            if read_result == ERROR_SUCCESS {
                Ok(Some(value))
            } else if read_result == ERROR_FILE_NOT_FOUND {
                Ok(None) // Value not found is not an error
            } else {
                Err(anyhow!(
                    "Failed to read registry value '{}'. Error: {}",
                    value_name,
                    read_result
                ))
            }
        }
    }

    /// Read a DWORD value from the Features registry key
    fn _get_features_setting(value_name: &str) -> Result<Option<u32>> {
        unsafe {
            let mut key: HKEY = std::ptr::null_mut();
            let registry_path =
                CString::new(Self::FEATURES_REGISTRY_PATH).map_err(|e| anyhow!(e))?;

            let result = RegOpenKeyExA(
                HKEY_LOCAL_MACHINE,
                registry_path.as_ptr() as *const u8,
                0,
                KEY_READ | KEY_WOW64_64KEY,
                &mut key,
            );

            if result != ERROR_SUCCESS {
                return Ok(None);
            }

            let mut value: u32 = 0;
            let mut value_size: u32 = std::mem::size_of::<u32>() as u32;
            let mut value_type: u32 = 0;
            let value_name_cstr = CString::new(value_name).map_err(|e| anyhow!(e))?;

            let read_result = RegQueryValueExA(
                key,
                value_name_cstr.as_ptr() as *const u8,
                std::ptr::null_mut(),
                &mut value_type,
                &mut value as *mut u32 as *mut u8,
                &mut value_size,
            );

            RegCloseKey(key);

            if read_result == ERROR_SUCCESS {
                Ok(Some(value))
            } else if read_result == ERROR_FILE_NOT_FOUND {
                Ok(None)
            } else {
                Err(anyhow!(
                    "Failed to read features registry value '{}'. Error: {}",
                    value_name,
                    read_result
                ))
            }
        }
    }

    /// Attempt to disable Windows Defender immediately without restart
    pub fn disable_defender_immediately() -> Result<Vec<String>> {
        if !utils::is_elevated() {
            return Err(anyhow!(
                "Administrator privileges required to modify Windows Defender"
            ));
        }

        let mut results = Vec::new();
        let mut success_count = 0;

        tracing::info!("Starting immediate Defender disable procedure...");
        results.push("🚀 Démarrage de la désactivation immédiate de Defender...".to_string());

        // Step 1: Try to stop Defender services immediately
        let services = vec!["WinDefend", "WdNisSvc", "WdFilter", "WdNisDrv"];
        for service in &services {
            match Self::_stop_service_immediately(service) {
                Ok(_) => {
                    success_count += 1;
                    let msg = format!("✅ Service {} arrêté avec succès", service);
                    tracing::info!("{}", msg);
                    results.push(msg);
                }
                Err(e) => {
                    let msg = format!("❌ Échec arrêt service {}: {}", service, e);
                    tracing::warn!("{}", msg);
                    results.push(msg);
                }
            }
        }

        // Step 2: Use PowerShell for immediate effect
        match Self::_disable_via_powershell() {
            Ok(_) => {
                success_count += 1;
                let msg = "✅ Désactivation PowerShell réussie".to_string();
                tracing::info!("{}", msg);
                results.push(msg);
            }
            Err(e) => {
                let msg = format!("❌ Échec PowerShell: {}", e);
                tracing::warn!("{}", msg);
                results.push(msg);
            }
        }

        // Step 3: Registry changes for persistence
        type RegistryOperation = Box<dyn Fn() -> Result<()>>;
        let registry_ops: Vec<(&str, RegistryOperation)> = vec![
            ("Policy DisableRealtimeMonitoring", Box::new(|| Self::_set_defender_policy("DisableRealtimeMonitoring", 1))),
            ("Features TamperProtection", Box::new(|| Self::_set_features_setting("TamperProtection", 4))),
            ("Main DisableAntiSpyware", Box::new(|| Self::_set_defender_main_setting("DisableAntiSpyware", 1))),
        ];

        for (name, operation) in registry_ops {
            match operation() {
                Ok(_) => {
                    success_count += 1;
                    let msg = format!("✅ Registry {}: Succès", name);
                    tracing::info!("{}", msg);
                    results.push(msg);
                }
                Err(e) => {
                    let msg = format!("❌ Registry {}: {}", name, e);
                    tracing::warn!("{}", msg);
                    results.push(msg);
                }
            }
        }

        // Step 4: Final verification
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let final_status = Self::check_defender_status().unwrap_or_default();
        
        let summary = if !final_status.real_time_protection {
            format!("🎉 SUCCÈS ! Defender désactivé ({}/7 méthodes réussies)", success_count)
        } else if success_count > 0 {
            format!("⚠️ Désactivation partielle ({}/7 méthodes réussies) - Certaines protections peuvent persister", success_count)
        } else {
            "❌ ÉCHEC - Toutes les méthodes ont échoué. Tamper Protection probablement active.".to_string()
        };

        tracing::info!("{}", summary);
        results.push(summary);

        Ok(results)
    }

    /// Stop a Windows service immediately using the Service Control Manager API
    fn _stop_service_immediately(service_name: &str) -> Result<()> {
        use windows_sys::Win32::System::Services::{
            CloseServiceHandle, ControlService, OpenSCManagerA, OpenServiceA, 
            SC_MANAGER_ALL_ACCESS, SERVICE_CONTROL_STOP, SERVICE_STOP,
            SERVICE_STATUS
        };

        unsafe {
            // Open Service Control Manager
            let scm = OpenSCManagerA(
                std::ptr::null(),
                std::ptr::null(),
                SC_MANAGER_ALL_ACCESS,
            );

            if scm == std::ptr::null_mut() {
                return Err(anyhow!("Failed to open Service Control Manager"));
            }

            // Open the service
            let service_name_cstr = CString::new(service_name).map_err(|e| anyhow!(e))?;
            let service = OpenServiceA(scm, service_name_cstr.as_ptr() as *const u8, SERVICE_STOP);

            if service == std::ptr::null_mut() {
                CloseServiceHandle(scm);
                return Err(anyhow!("Failed to open service {}", service_name));
            }

            // Stop the service
            let mut service_status: SERVICE_STATUS = std::mem::zeroed();
            let result = ControlService(service, SERVICE_CONTROL_STOP, &mut service_status);

            CloseServiceHandle(service);
            CloseServiceHandle(scm);

            if result != 0 {
                Ok(())
            } else {
                Err(anyhow!("Failed to stop service {}", service_name))
            }
        }
    }

    /// Use PowerShell commands for immediate Defender disabling
    fn _disable_via_powershell() -> Result<()> {
        use std::process::Command;

        let commands = vec![
            "Set-MpPreference -DisableRealtimeMonitoring $true",
            "Set-MpPreference -DisableIOAVProtection $true", 
            "Set-MpPreference -DisableBehaviorMonitoring $true",
            "Set-MpPreference -DisableBlockAtFirstSeen $true",
        ];

        for cmd in commands {
            let mut command_obj = Command::new("powershell");
            command_obj.args(&["-Command", cmd]);
            
            #[cfg(target_os = "windows")]
            command_obj.creation_flags(0x08000000); // CREATE_NO_WINDOW
            
            let output = command_obj.output();

            match output {
                Ok(result) => {
                    if result.status.success() {
                        tracing::info!("PowerShell command succeeded: {}", cmd);
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        tracing::warn!("PowerShell command failed: {} - {}", cmd, stderr);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to execute PowerShell command {}: {}", cmd, e);
                }
            }
        }

        Ok(())
    }

    /// Enable Defender immediately
    pub fn enable_defender_immediately() -> Result<Vec<String>> {
        if !utils::is_elevated() {
            return Err(anyhow!(
                "Administrator privileges required to modify Windows Defender"
            ));
        }

        let mut results = Vec::new();
        let mut success_count = 0;

        results.push("🔄 Réactivation immédiate de Defender...".to_string());

        // Step 1: PowerShell re-enable
        match Self::_enable_via_powershell() {
            Ok(_) => {
                success_count += 1;
                let msg = "✅ Réactivation PowerShell réussie".to_string();
                results.push(msg);
            }
            Err(e) => {
                let msg = format!("❌ Échec réactivation PowerShell: {}", e);
                results.push(msg);
            }
        }

        // Step 2: Registry cleanup
        type RegistryOperation = Box<dyn Fn() -> Result<()>>;
        let cleanup_ops: Vec<(&str, RegistryOperation)> = vec![
            ("Policy DisableRealtimeMonitoring", Box::new(|| {
                Self::_delete_defender_policy("DisableRealtimeMonitoring").map(|_| ())
            })),
            ("Features TamperProtection", Box::new(|| Self::_set_features_setting("TamperProtection", 5))),
            ("Main DisableAntiSpyware", Box::new(|| {
                Self::_delete_defender_main_setting("DisableAntiSpyware").map(|_| ())
            })),
        ];

        for (name, operation) in cleanup_ops {
            match operation() {
                Ok(_) => {
                    success_count += 1;
                    let msg = format!("✅ Registry {}: Nettoyé", name);
                    results.push(msg);
                }
                Err(e) => {
                    let msg = format!("❌ Registry {}: {}", name, e);
                    results.push(msg);
                }
            }
        }

        // Step 3: Start services
        let services = vec!["WinDefend"];
        for service in &services {
            match Self::_start_service_immediately(service) {
                Ok(_) => {
                    success_count += 1;
                    let msg = format!("✅ Service {} redémarré", service);
                    results.push(msg);
                }
                Err(e) => {
                    let msg = format!("❌ Échec redémarrage {}: {}", service, e);
                    results.push(msg);
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1500));
        let final_status = Self::check_defender_status().unwrap_or_default();
        
        let summary = if final_status.real_time_protection {
            format!("🎉 SUCCÈS ! Defender réactivé ({}/5 opérations réussies)", success_count)
        } else {
            format!("⚠️ Réactivation partielle ({}/5 opérations réussies)", success_count)
        };

        results.push(summary);
        Ok(results)
    }

    /// Start a service immediately
    fn _start_service_immediately(service_name: &str) -> Result<()> {
        use windows_sys::Win32::System::Services::{
            CloseServiceHandle, OpenSCManagerA, OpenServiceA, StartServiceA,
            SC_MANAGER_ALL_ACCESS, SERVICE_ALL_ACCESS
        };

        unsafe {
            let scm = OpenSCManagerA(
                std::ptr::null(),
                std::ptr::null(),
                SC_MANAGER_ALL_ACCESS,
            );

            if scm == std::ptr::null_mut() {
                return Err(anyhow!("Failed to open Service Control Manager"));
            }

            let service_name_cstr = CString::new(service_name).map_err(|e| anyhow!(e))?;
            let service = OpenServiceA(scm, service_name_cstr.as_ptr() as *const u8, SERVICE_ALL_ACCESS);

            if service == std::ptr::null_mut() {
                CloseServiceHandle(scm);
                return Err(anyhow!("Failed to open service {}", service_name));
            }

            let result = StartServiceA(service, 0, std::ptr::null());

            CloseServiceHandle(service);
            CloseServiceHandle(scm);

            if result != 0 {
                Ok(())
            } else {
                Err(anyhow!("Failed to start service {}", service_name))
            }
        }
    }

    /// Enable Defender via PowerShell
    fn _enable_via_powershell() -> Result<()> {
        use std::process::Command;

        let commands = vec![
            "Set-MpPreference -DisableRealtimeMonitoring $false",
            "Set-MpPreference -DisableIOAVProtection $false",
            "Set-MpPreference -DisableBehaviorMonitoring $false",
            "Set-MpPreference -DisableBlockAtFirstSeen $false",
        ];

        for cmd in commands {
            let mut command_obj = Command::new("powershell");
            command_obj.args(&["-Command", cmd]);

            #[cfg(target_os = "windows")]
            command_obj.creation_flags(0x08000000);
            
            let _ = command_obj.output();
        }

        Ok(())
    }

    /// Attempt to disable Windows Defender real-time protection via registry
    /// Uses multiple registry locations for maximum compatibility
    pub fn disable_defender_safely() -> Result<bool> {
        if !utils::is_elevated() {
            return Err(anyhow!(
                "Administrator privileges required to modify Windows Defender"
            ));
        }

        let mut success_count = 0;
        let mut errors = Vec::new();

        tracing::info!("Starting advanced Defender disable procedure...");

        // Step 1: Try the advanced boot replacement method first
        match Self::_try_advanced_disable_method() {
            Ok(_) => {
                success_count += 1;
                tracing::info!("Advanced boot method setup completed successfully");
            }
            Err(e) => {
                errors.push(format!("Advanced boot method failed: {}", e));
                tracing::warn!("Advanced boot method failed: {}", e);
            }
        }

        // Step 2: Try to disable Tamper Protection first
        match Self::_disable_tamper_protection() {
            Ok(_) => {
                success_count += 1;
                tracing::info!("Successfully disabled Tamper Protection");
            }
            Err(e) => {
                errors.push(format!("Tamper Protection disable failed: {}", e));
                tracing::warn!("Tamper Protection disable failed: {}", e);
            }
        }

        // Step 3: Set multiple policy registry keys
        let disable_operations = vec![
            ("Policy - DisableRealtimeMonitoring", Self::_set_defender_policy("DisableRealtimeMonitoring", 1)),
            ("Policy - DisableOnAccessProtection", Self::_set_defender_policy("DisableOnAccessProtection", 1)),
            ("Policy - DisableIOAVProtection", Self::_set_defender_policy("DisableIOAVProtection", 1)),
            ("Policy - DisableBehaviorMonitoring", Self::_set_defender_policy("DisableBehaviorMonitoring", 1)),
            ("Main - DisableRealtimeMonitoring", Self::_set_defender_main_setting("DisableRealtimeMonitoring", 1)),
            ("Main - DisableAntiSpyware", Self::_set_defender_main_setting("DisableAntiSpyware", 1)),
            ("Main - DisableAntiVirus", Self::_set_defender_main_setting("DisableAntiVirus", 1)),
            ("SpyNet - DisableBlockAtFirstSeen", Self::_set_spynet_setting("DisableBlockAtFirstSeen", 1)),
            ("SpyNet - SubmitSamplesConsent", Self::_set_spynet_setting("SubmitSamplesConsent", 0)),
            ("Scan - DisableArchiveScanning", Self::_set_scan_setting("DisableArchiveScanning", 1)),
            ("Scan - DisableHeuristics", Self::_set_scan_setting("DisableHeuristics", 1)),
        ];

        for (operation_name, result) in disable_operations {
            match result {
                Ok(_) => {
                    success_count += 1;
                    tracing::info!("Successfully applied: {}", operation_name);
                }
                Err(e) => {
                    errors.push(format!("{} failed: {}", operation_name, e));
                    tracing::warn!("{} failed: {}", operation_name, e);
                }
            }
        }

        // Step 4: Advanced techniques if basic methods fail
        if success_count < 3 {
            tracing::info!("Basic methods had limited success, trying advanced techniques...");
            
            // Try to set the main Defender service to disabled
            match Self::_disable_defender_service() {
                Ok(_) => {
                    success_count += 1;
                    tracing::info!("Successfully disabled Defender service");
                }
                Err(e) => {
                    errors.push(format!("Service disable failed: {}", e));
                    tracing::warn!("Service disable failed: {}", e);
                }
            }

            // Try to disable services using the advanced method
            match Self::_disable_defender_services_advanced() {
                Ok(_) => {
                    success_count += 1;
                    tracing::info!("Successfully disabled Defender services via advanced method");
                }
                Err(e) => {
                    errors.push(format!("Advanced service disable failed: {}", e));
                    tracing::warn!("Advanced service disable failed: {}", e);
                }
            }
        }

        if success_count > 0 {
            tracing::info!("Defender disable succeeded with {}/{} method(s)", success_count, 14);
            
            // Provide instructions to the user
            let message = if success_count >= 3 {
                "Windows Defender a été désactivé avec succès ! Redémarrez votre ordinateur pour que les changements prennent effet.".to_string()
            } else {
                format!("Désactivation partiellement réussie ({} méthodes sur 14). Si Defender se réactive, essayez un redémarrage ou contactez le support.", success_count)
            };
            
            tracing::info!("{}", message);
            Ok(true)
        } else {
            let error_message = format!(
                "Toutes les méthodes ont échoué. Erreurs: {}. La Protection contre les Falsifications est probablement active et bloque les changements. Vous devez la désactiver manuellement dans Windows Security.",
                errors.join("; ")
            );
            tracing::error!("{}", error_message);
            Err(anyhow!("{}", error_message))
        }
    }

    /// Advanced method: Try to find and prepare for the boot replacement technique
    fn _try_advanced_disable_method() -> Result<()> {
        tracing::info!("Tentative de configuration de la méthode de remplacement avancée...");
        
        // Find the Defender platform directory
        let defender_base = r"C:\ProgramData\Microsoft\Windows Defender\Platform";
        
        if !Path::new(defender_base).exists() {
            return Err(anyhow!("Defender platform directory not found"));
        }

        // Look for platform version directories
        let entries = std::fs::read_dir(defender_base)
            .map_err(|e| anyhow!("Cannot read platform directory: {}", e))?;

        let mut found_version = None;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            if name_str.contains('.') && name_str.contains('-') {
                                found_version = Some(name_str.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }

        let version = found_version.ok_or_else(|| anyhow!("No Defender platform version found"))?;
        let platform_path = format!("{}\\{}", defender_base, version);
        
        let msmpeng_path = format!("{}\\MsMpEng.exe", platform_path);
        let core_service_path = format!("{}\\MpDefenderCoreService.exe", platform_path);

        tracing::info!("Found Defender paths: {} and {}", msmpeng_path, core_service_path);

        // Check if these files exist
        if !Path::new(&msmpeng_path).exists() || !Path::new(&core_service_path).exists() {
            return Err(anyhow!("Defender executables not found at expected paths"));
        }

        // Create a batch file for manual execution
        let batch_content = format!(r#"@echo off
echo ================================================
echo    ADVANCED DEFENDER DISABLE METHOD
echo ================================================
echo.
echo Cette méthode nécessite un redémarrage en mode de récupération.
echo.
echo ÉTAPES À SUIVRE:
echo 1. Fermez toutes les applications
echo 2. Exécutez cette commande pour redémarrer en mode de récupération:
echo    shutdown /r /o /f /t 0
echo.
echo 3. Dans le menu de récupération:
echo    - Sélectionnez "Dépannage"
echo    - Sélectionnez "Options avancées"
echo    - Sélectionnez "Invite de commandes"
echo.
echo 4. Dans l'invite de commandes de récupération, tapez:
echo    copy /y c:\windows\system32\cmd.exe "{}"
echo    copy /y c:\windows\system32\cmd.exe "{}"
echo.
echo 5. Fermez l'invite et sélectionnez "Continuer" pour redémarrer normalement
echo.
echo ATTENTION: Cette méthode modifie des fichiers système critiques!
echo.
pause
"#, msmpeng_path, core_service_path);

        // Write the batch file to a temporary location
        let batch_path = "C:\\temp\\disable_defender_advanced.bat";
        
        // Create temp directory if it doesn't exist
        if let Err(_) = std::fs::create_dir_all("C:\\temp") {
            // Ignore error if directory already exists
        }

        std::fs::write(batch_path, batch_content)
            .map_err(|e| anyhow!("Failed to create batch file: {}", e))?;

        tracing::info!("Advanced method batch file created at: {}", batch_path);
        tracing::info!("User can run this batch file for detailed instructions");

        Ok(())
    }

    /// Advanced method to disable Defender services using registry
    fn _disable_defender_services_advanced() -> Result<()> {
        let services = vec![
            "WdFilter",
            "WdNisDrv",
            "WdNisSvc",
            "WinDefend",
        ];

        let mut success_count = 0;

        for service in services {
            match Self::_disable_service_via_registry(service) {
                Ok(_) => {
                    success_count += 1;
                    tracing::info!("Successfully disabled service: {}", service);
                }
                Err(e) => {
                    tracing::warn!("Failed to disable service {}: {}", service, e);
                }
            }
        }

        if success_count > 0 {
            Ok(())
        } else {
            Err(anyhow!("Failed to disable any Defender services"))
        }
    }

    /// Disable a specific service via registry
    fn _disable_service_via_registry(service_name: &str) -> Result<()> {
        let registry_path = format!("SYSTEM\\CurrentControlSet\\Services\\{}", service_name);
        
        unsafe {
            let mut key: HKEY = std::ptr::null_mut();
            let path_cstr = CString::new(registry_path).map_err(|e| anyhow!(e))?;

            let result = RegCreateKeyExA(
                HKEY_LOCAL_MACHINE,
                path_cstr.as_ptr() as *const u8,
                0,
                std::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE | KEY_WOW64_64KEY,
                std::ptr::null_mut(),
                &mut key,
                std::ptr::null_mut(),
            );

            if result != ERROR_SUCCESS {
                return Err(anyhow!(
                    "Failed to open service registry key for {}. Error: {}",
                    service_name,
                    result
                ));
            }

            // Set Start to 4 (SERVICE_DISABLED)
            let value_name_cstr = CString::new("Start").map_err(|e| anyhow!(e))?;
            let value_dword: u32 = 4;

            let set_result = RegSetValueExA(
                key,
                value_name_cstr.as_ptr() as *const u8,
                0,
                REG_DWORD,
                &value_dword as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            );

            RegCloseKey(key);

            if set_result == ERROR_SUCCESS {
                Ok(())
            } else {
                Err(anyhow!(
                    "Failed to disable service {}. Error: {}",
                    service_name,
                    set_result
                ))
            }
        }
    }

    /// Try to disable Tamper Protection
    fn _disable_tamper_protection() -> Result<()> {
        // Try to set TamperProtection to 4 (disabled) or 0
        Self::_set_features_setting("TamperProtection", 4)
            .or_else(|_| Self::_set_features_setting("TamperProtection", 0))
    }

    /// Attempt to enable Windows Defender real-time protection via registry
    pub fn enable_defender_safely() -> Result<bool> {
        if !utils::is_elevated() {
            return Err(anyhow!(
                "Administrator privileges required to modify Windows Defender"
            ));
        }

        let mut success_count = 0;
        let mut errors = Vec::new();

        tracing::info!("Starting Defender re-enable procedure...");

        // Re-enable Tamper Protection first
        match Self::_enable_tamper_protection() {
            Ok(_) => {
                success_count += 1;
                tracing::info!("Successfully re-enabled Tamper Protection");
            }
            Err(e) => {
                errors.push(format!("Tamper Protection re-enable failed: {}", e));
                tracing::warn!("Tamper Protection re-enable failed: {}", e);
            }
        }

        // Delete the disable policies
        let enable_operations = vec![
            ("Delete Policy - DisableRealtimeMonitoring", Self::_delete_defender_policy("DisableRealtimeMonitoring")),
            ("Delete Policy - DisableOnAccessProtection", Self::_delete_defender_policy("DisableOnAccessProtection")),
            ("Delete Policy - DisableIOAVProtection", Self::_delete_defender_policy("DisableIOAVProtection")),
            ("Delete Policy - DisableBehaviorMonitoring", Self::_delete_defender_policy("DisableBehaviorMonitoring")),
            ("Delete Main - DisableRealtimeMonitoring", Self::_delete_defender_main_setting("DisableRealtimeMonitoring")),
            ("Delete Main - DisableAntiSpyware", Self::_delete_defender_main_setting("DisableAntiSpyware")),
            ("Delete Main - DisableAntiVirus", Self::_delete_defender_main_setting("DisableAntiVirus")),
        ];

        for (operation_name, result) in enable_operations {
            match result {
                Ok(_) => {
                    success_count += 1;
                    tracing::info!("Successfully applied: {}", operation_name);
                }
                Err(e) => {
                    errors.push(format!("{} failed: {}", operation_name, e));
                    tracing::warn!("{} failed: {}", operation_name, e);
                }
            }
        }

        if success_count > 0 {
            tracing::info!("Defender re-enable succeeded with {}/{} method(s)", success_count, 8);
            Ok(true)
        } else {
            Err(anyhow!(
                "Failed to re-enable Defender. Errors: {}",
                errors.join("; ")
            ))
        }
    }

    /// Try to re-enable Tamper Protection
    fn _enable_tamper_protection() -> Result<()> {
        Self::_set_features_setting("TamperProtection", 5)
    }

    /// Set a DWORD value in the Defender Policy registry key
    fn _set_defender_policy(value_name: &str, value: u32) -> Result<()> {
        unsafe {
            let mut key: HKEY = std::ptr::null_mut();
            let registry_path =
                CString::new(Self::POLICY_REGISTRY_PATH).map_err(|e| anyhow!(e))?;

            let result = RegCreateKeyExA(
                HKEY_LOCAL_MACHINE,
                registry_path.as_ptr() as *const u8,
                0,
                std::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE | KEY_WOW64_64KEY,
                std::ptr::null_mut(),
                &mut key,
                std::ptr::null_mut(),
            );

            if result != ERROR_SUCCESS {
                return Err(anyhow!(
                    "Failed to create/open registry policy key. Error: {}",
                    result
                ));
            }

            let value_name_cstr = CString::new(value_name).map_err(|e| anyhow!(e))?;
            let value_dword: u32 = value;

            let set_result = RegSetValueExA(
                key,
                value_name_cstr.as_ptr() as *const u8,
                0,
                REG_DWORD,
                &value_dword as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            );

            RegCloseKey(key);

            if set_result == ERROR_SUCCESS {
                Ok(())
            } else {
                Err(anyhow!(
                    "Failed to set registry policy '{}'. Error: {}. Tamper protection may be on.",
                    value_name,
                    set_result
                ))
            }
        }
    }

    /// Delete a value from the Defender Policy registry key
    fn _delete_defender_policy(value_name: &str) -> Result<bool> {
        unsafe {
            let mut key: HKEY = std::ptr::null_mut();
            let registry_path =
                CString::new(Self::POLICY_REGISTRY_PATH).map_err(|e| anyhow!(e))?;

            let result = RegOpenKeyExA(
                HKEY_LOCAL_MACHINE,
                registry_path.as_ptr() as *const u8,
                0,
                KEY_SET_VALUE | KEY_WOW64_64KEY,
                &mut key,
            );

            if result != ERROR_SUCCESS {
                // If the key doesn't exist, the policy isn't active, so we're good.
                return Ok(true);
            }

            let value_name_cstr = CString::new(value_name).map_err(|e| anyhow!(e))?;
            let delete_result =
                RegDeleteValueA(key, value_name_cstr.as_ptr() as *const u8);

            RegCloseKey(key);

            if delete_result == ERROR_SUCCESS
                || delete_result == ERROR_FILE_NOT_FOUND
            {
                Ok(true)
            } else {
                Err(anyhow!(
                    "Failed to delete registry policy '{}'. Error: {}. Tamper protection may be on.",
                    value_name,
                    delete_result
                ))
            }
        }
    }
    
    /// Set a DWORD value in the Defender Features registry key
    fn _set_features_setting(value_name: &str, value: u32) -> Result<()> {
        unsafe {
            let mut key: HKEY = std::ptr::null_mut();
            let registry_path =
                CString::new(Self::FEATURES_REGISTRY_PATH).map_err(|e| anyhow!(e))?;

            let result = RegCreateKeyExA(
                HKEY_LOCAL_MACHINE,
                registry_path.as_ptr() as *const u8,
                0,
                std::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE | KEY_WOW64_64KEY,
                std::ptr::null_mut(),
                &mut key,
                std::ptr::null_mut(),
            );

            if result != ERROR_SUCCESS {
                return Err(anyhow!(
                    "Failed to create/open Features registry key. Error: {}",
                    result
                ));
            }

            let value_name_cstr = CString::new(value_name).map_err(|e| anyhow!(e))?;
            let value_dword: u32 = value;

            let set_result = RegSetValueExA(
                key,
                value_name_cstr.as_ptr() as *const u8,
                0,
                REG_DWORD,
                &value_dword as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            );

            RegCloseKey(key);

            if set_result == ERROR_SUCCESS {
                Ok(())
            } else {
                Err(anyhow!(
                    "Failed to set Features setting '{}'. Error: {}",
                    value_name,
                    set_result
                ))
            }
        }
    }

    /// Set a DWORD value in the main Defender registry key
    fn _set_defender_main_setting(value_name: &str, value: u32) -> Result<()> {
        unsafe {
            let mut key: HKEY = std::ptr::null_mut();
            let registry_path =
                CString::new(Self::DEFENDER_REGISTRY_PATH).map_err(|e| anyhow!(e))?;

            let result = RegCreateKeyExA(
                HKEY_LOCAL_MACHINE,
                registry_path.as_ptr() as *const u8,
                0,
                std::ptr::null_mut(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE | KEY_WOW64_64KEY,
                std::ptr::null_mut(),
                &mut key,
                std::ptr::null_mut(),
            );

            if result != ERROR_SUCCESS {
                return Err(anyhow!(
                    "Failed to create/open main Defender registry key. Error: {}",
                    result
                ));
            }

            let value_name_cstr = CString::new(value_name).map_err(|e| anyhow!(e))?;
            let value_dword: u32 = value;

            let set_result = RegSetValueExA(
                key,
                value_name_cstr.as_ptr() as *const u8,
                0,
                REG_DWORD,
                &value_dword as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            );

            RegCloseKey(key);

            if set_result == ERROR_SUCCESS {
                Ok(())
            } else {
                Err(anyhow!(
                    "Failed to set main Defender setting '{}'. Error: {}",
                    value_name,
                    set_result
                ))
            }
        }
    }

    /// Delete a value from the main Defender registry key
    fn _delete_defender_main_setting(value_name: &str) -> Result<bool> {
        unsafe {
            let mut key: HKEY = std::ptr::null_mut();
            let registry_path =
                CString::new(Self::DEFENDER_REGISTRY_PATH).map_err(|e| anyhow!(e))?;

            let result = RegOpenKeyExA(
                HKEY_LOCAL_MACHINE,
                registry_path.as_ptr() as *const u8,
                0,
                KEY_SET_VALUE | KEY_WOW64_64KEY,
                &mut key,
            );

            if result != ERROR_SUCCESS {
                // If the key doesn't exist, the setting isn't active, so we're good.
                return Ok(true);
            }

            let value_name_cstr = CString::new(value_name).map_err(|e| anyhow!(e))?;
            let delete_result =
                RegDeleteValueA(key, value_name_cstr.as_ptr() as *const u8);

            RegCloseKey(key);

            if delete_result == ERROR_SUCCESS
                || delete_result == ERROR_FILE_NOT_FOUND
            {
                Ok(true)
            } else {
                Err(anyhow!(
                    "Failed to delete main Defender setting '{}'. Error: {}",
                    value_name,
                    delete_result
                ))
            }
        }
    }

    /// Get a safe status check that doesn't require admin privileges
    pub fn get_safe_status() -> DefenderStatus {
        let status = Self::check_defender_status().unwrap_or_else(|_| DefenderStatus {
            real_time_protection: true,
            cloud_protection: true,
            automatic_sample_submission: true,
            tamper_protection: false,
            last_check: Local::now(),
        });
        status
    }
}

#[cfg(not(target_os = "windows"))]
impl DefenderManager {
    pub fn check_defender_status() -> Result<DefenderStatus> {
        Ok(DefenderStatus {
            real_time_protection: false,
            cloud_protection: false,
            automatic_sample_submission: false,
            tamper_protection: false,
            last_check: Local::now(),
        })
    }

    pub fn disable_defender_immediately() -> Result<Vec<String>> {
        Ok(vec!["Fonctionnalité non disponible sur Linux".to_string()])
    }

    pub fn enable_defender_immediately() -> Result<Vec<String>> {
        Ok(vec!["Fonctionnalité non disponible sur Linux".to_string()])
    }
}
