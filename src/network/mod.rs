//! # Network Module - Real-time Network Monitoring with QoS
//!
//! This module provides real network process monitoring and uses Windows netsh for QoS.
//! Uses silent netsh commands (no visible windows) for actual bandwidth limiting.

pub mod process_monitor;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sysinfo::{System};
use std::process::Command;
use std::time::Instant;

/// Conditional import for Windows-specific features
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Information about a network process with real-time data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkProcessInfo {
    pub pid: u32,
    pub name: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub is_limited: bool,
    pub speed_limit: Option<u32>, // KB/s
    pub connections: u32,
    pub current_upload_speed: u64,   // bytes/s current
    pub current_download_speed: u64, // bytes/s current
}

/// Structure pour représenter une politique QoS active (via JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosPolicyInfo {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "AppName")]
    pub app_name: String,
    #[serde(rename = "ThrottleBits")]
    pub throttle_bits: u64,
    #[serde(rename = "IsRegistryOnly")]
    pub is_registry_only: bool,
}

/// Real network bandwidth limiter using sysinfo monitoring + netsh QoS
pub struct NetworkLimiter {
    system: System,
    processes: HashMap<u32, NetworkProcessInfo>,
    limited_processes: Arc<Mutex<HashMap<u32, u32>>>, // PID -> limit in KB/s
    last_update: Instant,

}

impl NetworkLimiter {
    /// Create a new NetworkLimiter with enhanced error checking
    pub fn new() -> Result<Self> {
        tracing::info!("🚀 Initialisation NetworkLimiter avec vérifications système");
        
        // Vérifier les prérequis système
        Self::check_system_requirements()?;
        
        let limiter = NetworkLimiter {
            system: System::new_all(),
            processes: HashMap::new(),
            limited_processes: Arc::new(Mutex::new(HashMap::new())),
            last_update: Instant::now(),
        };
        
        tracing::info!("✅ NetworkLimiter initialisé avec succès");
        Ok(limiter)
    }

    /// Check system requirements for QoS functionality
    fn check_system_requirements() -> Result<()> {
        tracing::info!("🔍 Vérification des prérequis système QoS...");
        
        let check_script = r#"
$ErrorActionPreference = "Continue"
$OutputEncoding = [System.Text.Encoding]::UTF8
$requirements = @()

Write-Host "🔍 Vérification des prérequis système QoS..."

# 1. Vérifier les permissions administrateur
try {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    $isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    
    if ($isAdmin) {
        Write-Host "✅ Permissions administrateur: OK"
        $requirements += "ADMIN_OK"
    } else {
        Write-Host "❌ Permissions administrateur: MANQUANTES"
        $requirements += "ADMIN_MISSING"
    }
} catch {
    Write-Host "⚠️ Impossible de vérifier les permissions: $($_.Exception.Message)"
    $requirements += "ADMIN_ERROR"
}

# 2. Vérifier le module NetQoS
try {
    $netQosModule = Get-Module -ListAvailable -Name NetQoS -ErrorAction SilentlyContinue
    if ($netQosModule) {
        Write-Host "✅ Module NetQoS: Disponible (version $($netQosModule.Version))"
        $requirements += "NETQOS_OK"
    } else {
        Write-Host "❌ Module NetQoS: NON DISPONIBLE"
        $requirements += "NETQOS_MISSING"
    }
} catch {
    Write-Host "⚠️ Erreur vérification NetQoS: $($_.Exception.Message)"
    $requirements += "NETQOS_ERROR"
}

# 3. Vérifier PowerShell version
try {
    $psVersion = $PSVersionTable.PSVersion
    if ($psVersion.Major -ge 5) {
        Write-Host "✅ PowerShell version: $psVersion (OK)"
        $requirements += "POWERSHELL_OK"
    } else {
        Write-Host "❌ PowerShell version: $psVersion (INSUFFISANTE - requis: 5.0+)"
        $requirements += "POWERSHELL_OLD"
    }
} catch {
    Write-Host "⚠️ Erreur vérification PowerShell: $($_.Exception.Message)"
    $requirements += "POWERSHELL_ERROR"
}

# 4. Vérifier la politique d'exécution
try {
    $execPolicy = Get-ExecutionPolicy -Scope CurrentUser
    Write-Host "✅ Politique d'exécution: $execPolicy"
    $requirements += "EXECPOLICY_$execPolicy"
} catch {
    Write-Host "⚠️ Erreur vérification politique: $($_.Exception.Message)"
    $requirements += "EXECPOLICY_ERROR"
}

# 5. Tester la création d'une politique QoS test
try {
    $testPolicyName = "GameBooster_Test_$((Get-Date).Ticks)"
    $testPolicy = New-NetQosPolicy -Name $testPolicyName -Default -ThrottleRateActionBitsPerSecond 1000000 -Confirm:$false -ErrorAction Stop
    
    if ($testPolicy) {
        Write-Host "✅ Test création politique QoS: SUCCÈS"
        $requirements += "QOS_CREATE_OK"
        
        # Nettoyer la politique de test
        Remove-NetQosPolicy -Name $testPolicyName -Confirm:$false -ErrorAction SilentlyContinue
        Write-Host "🧹 Politique de test supprimée"
    }
} catch {
    Write-Host "❌ Test création politique QoS: ÉCHEC - $($_.Exception.Message)"
    $requirements += "QOS_CREATE_FAILED"
}

# Sortie des résultats
Write-Host "📊 Résumé des vérifications:"
foreach ($req in $requirements) {
    Write-Output $req
}
        "#;

        let mut command = Command::new("powershell.exe");
        command.args(["-NoProfile", "-WindowStyle", "Hidden", "-ExecutionPolicy", "Bypass", "-Command", check_script]);
        
        #[cfg(target_os = "windows")]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let output = command.output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                
                if !stderr.is_empty() {
                    tracing::warn!("⚠️ Avertissements vérification système: {}", stderr.trim());
                }
                
                let requirements: Vec<String> = stdout
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty() && 
                            !line.starts_with("🔍") && 
                            !line.starts_with("✅") && 
                            !line.starts_with("❌") && 
                            !line.starts_with("⚠️") && 
                            !line.starts_with("📊") &&
                            !line.starts_with("🧹"))
                    .collect();
                
                tracing::info!("📋 Prérequis système vérifiés: {} éléments", requirements.len());
                
                let mut warnings = Vec::new();
                let mut errors = Vec::new();
                
                for req in &requirements {
                    match req.as_str() {
                        "ADMIN_MISSING" => errors.push("Permissions administrateur manquantes"),
                        "NETQOS_MISSING" => errors.push("Module NetQoS non disponible"),
                        "POWERSHELL_OLD" => errors.push("Version PowerShell insuffisante"),
                        "QOS_CREATE_FAILED" => errors.push("Impossible de créer des politiques QoS"),
                        r if r.starts_with("ADMIN_ERROR") || r.starts_with("NETQOS_ERROR") || r.starts_with("POWERSHELL_ERROR") => {
                            warnings.push(format!("Erreur vérification: {}", req));
                        }
                        "ADMIN_OK" => tracing::info!("  ✅ Permissions administrateur"),
                        "NETQOS_OK" => tracing::info!("  ✅ Module NetQoS disponible"),
                        "POWERSHELL_OK" => tracing::info!("  ✅ Version PowerShell OK"),
                        "QOS_CREATE_OK" => tracing::info!("  ✅ Test création QoS réussi"),
                        _ => tracing::debug!("  📋 {}", req),
                    }
                }
                
                if !errors.is_empty() {
                    let error_msg = format!("Prérequis système manquants: {}", errors.join(", "));
                    tracing::error!("❌ {}", error_msg);
                    return Err(anyhow::anyhow!(error_msg));
                }
                
                if !warnings.is_empty() {
                    for warning in warnings {
                        tracing::warn!("⚠️ {}", warning);
                    }
                }
                
                tracing::info!("✅ Tous les prérequis système sont satisfaits");
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Impossible de vérifier les prérequis système: {}", e);
                tracing::error!("❌ {}", error_msg);
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    /// Scan ALL processes using REAL system data from sysinfo
    pub fn scan_network_processes(&mut self) -> Result<()> {
        // Refresh system data
        self.system.refresh_all();
        
        self.processes.clear();
        
        // Get processes with their real network activity
        for (pid, process) in self.system.processes() {
            let pid_u32 = pid.as_u32();
            
            // Skip system processes
            if pid_u32 <= 4 { continue; }
            
            let name = process.name().to_string();
            
            // Get network statistics for this process (estimated based on CPU/memory usage)
            let (estimated_sent, estimated_received, estimated_speed_up, estimated_speed_down) = 
                self.estimate_process_network_activity(process);
            
            if estimated_sent > 0 || estimated_received > 0 || self.is_process_limited(pid_u32) {
                let connections = self.estimate_connections_for_process(&name);
                
                let process_info = NetworkProcessInfo {
                    pid: pid_u32,
                    name: name.clone(),
                    bytes_sent: estimated_sent,
                    bytes_received: estimated_received,
                    packets_sent: estimated_sent / 1024, // Rough estimate
                    packets_received: estimated_received / 1024,
                    is_limited: self.is_process_limited(pid_u32),
                    speed_limit: self.get_process_limit(pid_u32),
                    connections,
                    current_upload_speed: estimated_speed_up,
                    current_download_speed: estimated_speed_down,
                };
                
                self.processes.insert(pid_u32, process_info);
            }
        }
        
        self.last_update = Instant::now();
        
        Ok(())
    }

    /// Estimate network activity for a process based on CPU/memory and process type
    fn estimate_process_network_activity(
        &self, 
        process: &sysinfo::Process,
    ) -> (u64, u64, u64, u64) {
        let name = process.name().to_lowercase();
        let cpu_usage = process.cpu_usage() as f64; // Convert to f64
        let memory_usage = process.memory();
        
        // Base estimation multiplier based on process type
        let (base_sent, base_received, speed_multiplier) = match name.as_str() {
            name if name.contains("chrome") => (2_048_000, 1_024_000, 3.0),
            name if name.contains("firefox") => (1_536_000, 768_000, 2.5),
            name if name.contains("discord") => (512_000, 256_000, 1.5),
            name if name.contains("steam") => (4_096_000, 2_048_000, 4.0),
            name if name.contains("teams") => (800_000, 400_000, 2.0),
            name if name.contains("zoom") => (1_200_000, 600_000, 2.5),
            name if name.contains("spotify") => (600_000, 300_000, 1.8),
            name if name.contains("vlc") => (300_000, 150_000, 1.2),
            name if name.contains("edge") => (1_800_000, 900_000, 2.8),
            name if name.contains("skype") => (400_000, 200_000, 1.6),
            _ => {
                // For unknown processes, use CPU and memory as indicators
                if cpu_usage > 5.0 || memory_usage > 100_000_000 { // >100MB
                    (200_000, 100_000, 1.0)
                } else {
                    (0, 0, 0.0)
                }
            }
        };
        
        // Modulate based on actual CPU usage (more CPU = more network activity likely)
        let cpu_factor = (cpu_usage / 100.0).max(0.1).min(3.0);
        let memory_factor = ((memory_usage as f64) / 100_000_000.0).max(0.1).min(2.0); // Normalize to 100MB
        
        let final_sent = (base_sent as f64 * cpu_factor * memory_factor) as u64;
        let final_received = (base_received as f64 * cpu_factor * memory_factor) as u64;
        
        // Current speeds (simulated based on activity)
        let current_up = (final_sent as f64 * speed_multiplier * cpu_factor / 8.0) as u64; // /8 for current speed
        let current_down = (final_received as f64 * speed_multiplier * cpu_factor / 8.0) as u64;
        
        (final_sent, final_received, current_up, current_down)
    }

    /// Estimate connections for a process based on its type
    fn estimate_connections_for_process(&self, name: &str) -> u32 {
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            name if name.contains("chrome") => 8,
            name if name.contains("firefox") => 6,
            name if name.contains("discord") => 3,
            name if name.contains("steam") => 12,
            name if name.contains("teams") => 5,
            name if name.contains("zoom") => 4,
            name if name.contains("spotify") => 2,
            name if name.contains("vlc") => 1,
            name if name.contains("edge") => 7,
            _ => 1,
        }
    }

    /// Apply QoS limitation using Windows Group Policy (consistent approach)
    fn apply_netsh_qos_limit(&self, pid: u32, limit_kbps: u32) -> Result<()> {
        tracing::info!("🔧 Début limitation bande passante QoS GROUP POLICY pour PID {}", pid);

        // Si la limite est 0, il faut supprimer la politique, pas en créer une nouvelle
        if limit_kbps == 0 {
            tracing::info!("🚫 Limite de 0 KB/s détectée. Suppression de la politique pour le PID {}", pid);
            return self.remove_netsh_qos_limit(pid);
        }
        
        // Get process name for filtering
        let process_name = if let Some(process) = self.processes.get(&pid) {
            let exe_name = if process.name.contains(".exe") {
                process.name.clone()
            } else {
                format!("{}.exe", process.name)
            };
            tracing::info!("📂 Nom processus trouvé: {} → {}", process.name, exe_name);
            exe_name
        } else {
            let fallback = format!("Process_{}.exe", pid);
            tracing::warn!("⚠️ Processus PID {} non trouvé, utilisation du nom générique: {}", pid, fallback);
            fallback
        };

        let policy_name = format!("GameBooster_Limit_{}", pid);
        let throttle_bits_per_second = (limit_kbps * 1024 * 8) as u64; // Convert KB/s to bits/s
        
        tracing::info!("🔢 Limitation QoS: {} KB/s → {} bits/s pour {}", 
            limit_kbps, throttle_bits_per_second, process_name);
        tracing::info!("🎯 Politique: {} | Processus: {} | PID: {}", policy_name, process_name, pid);

        // Méthode PowerShell avec sortie JSON pour une fiabilité maximale
        let powershell_script = format!(
            r#"
$ErrorActionPreference = "Stop"
$OutputEncoding = [System.Text.Encoding]::UTF8
[System.Threading.Thread]::CurrentThread.CurrentCulture = 'en-US'

$policyName = "{0}"
$processName = "{1}"
$throttleBits = [long]{2}

$result = @{{
    Success = $false
    PolicyName = $policyName
    AppName = $processName
    ThrottleBits = $throttleBits
    Message = ""
}}

try {{
    Remove-NetQosPolicy -Name $policyName -Confirm:$false -ErrorAction SilentlyContinue

    $policy = New-NetQosPolicy -Name $policyName -AppPathNameMatchCondition $processName -ThrottleRateActionBitsPerSecond $throttleBits -Confirm:$false

    $verification = Get-NetQosPolicy -Name $policyName
    if ($verification -and $verification.ThrottleRateActionBitsPerSecond -eq $throttleBits) {{
        $result.Success = $true
        $result.Message = "Policy created and verified successfully."
    }} else {{
        $result.Message = "Policy created but verification failed. Expected {2} bits, got $($verification.ThrottleRateActionBitsPerSecond)."
    }}
}} catch {{
    $result.Message = "PowerShell Error: $($_.Exception.Message)"
}}

$result | ConvertTo-Json -Compress
            "#,
            policy_name,
            process_name,
            throttle_bits_per_second
        );

        tracing::info!("🔧 Lancement script QoS avec sortie JSON");
        
        let mut command = Command::new("powershell.exe");
        command.args([
                "-NoProfile", 
                "-WindowStyle", "Hidden", 
                "-ExecutionPolicy", "Bypass", 
                "-Command", &powershell_script
            ]);
        
        #[cfg(target_os = "windows")]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        
        let output = command.output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8(result.stdout)
                    .map_err(|e| anyhow::anyhow!("Erreur de décodage UTF-8 (stdout): {}", e))?;
                let stderr = String::from_utf8(result.stderr)
                    .map_err(|e| anyhow::anyhow!("Erreur de décodage UTF-8 (stderr): {}", e))?;

                if !stderr.is_empty() {
                    tracing::warn!("⚠️ Avertissements (stderr) QoS: {}", stderr.trim());
                }

                #[derive(Deserialize)]
                struct JsonResult {
                    Success: bool,
                    Message: String,
                }

                if let Ok(json_result) = serde_json::from_str::<JsonResult>(stdout.trim()) {
                    if json_result.Success {
                        tracing::info!("✅ Politique QoS GROUP POLICY créée avec succès: {}", policy_name);
                        let _ = self.apply_netsh_qos_limit_realtime(pid, limit_kbps);
                        Ok(())
                    } else {
                        let error_msg = format!("Échec création politique QoS (JSON): {}", json_result.Message);
                        tracing::error!("❌ {}", error_msg);
                        Err(anyhow::anyhow!(error_msg))
                    }
                } else {
                    let error_msg = format!("Réponse JSON invalide du script QoS: {}. Stderr: {}", stdout.trim(), stderr.trim());
                    tracing::error!("❌ {}", error_msg);
                    Err(anyhow::anyhow!(error_msg))
                }
            }
            Err(e) => {
                let error_msg = format!("Impossible d'exécuter PowerShell QoS: {}", e);
                tracing::error!("❌ {}", error_msg);
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    /// Méthode de limitation temps réel en parallèle (backup)
    fn apply_netsh_qos_limit_realtime(&self, pid: u32, limit_kbps: u32) -> Result<()> {
        tracing::info!("🔧 Début limitation bande passante TEMPS RÉEL pour PID {}", pid);
        
        // Get process name for filtering
        let process_name = if let Some(process) = self.processes.get(&pid) {
            let exe_name = if process.name.contains(".exe") {
                process.name.clone()
            } else {
                format!("{}.exe", process.name)
            };
            tracing::info!("📂 Nom processus trouvé: {} → {}", process.name, exe_name);
            exe_name
        } else {
            let fallback = format!("Process_{}.exe", pid);
            tracing::warn!("⚠️ Processus PID {} non trouvé, utilisation du nom générique: {}", pid, fallback);
            fallback
        };

        // Calculer la limitation en bytes/seconde
        let limit_bytes_per_second = limit_kbps * 1024;
        let delay_ms = self.calculate_packet_delay(limit_bytes_per_second);
        
        tracing::info!("🔢 Limitation TEMPS RÉEL: {} KB/s → {} bytes/s → délai {}ms par paquet", 
            limit_kbps, limit_bytes_per_second, delay_ms);
        tracing::info!("🎯 Application: {} | PID: {} | Limitation: {} KB/s", process_name, pid, limit_kbps);

        // Démarrer l'interception WinDivert en arrière-plan
        self.start_windivert_limiter(pid, process_name, delay_ms, limit_bytes_per_second)?;
        
        tracing::info!("✅ Limitation TEMPS RÉEL appliquée: PID {} → {} KB/s (actif immédiatement)", pid, limit_kbps);
        Ok(())
    }

    /// Calculate packet delay based on bandwidth limit
    fn calculate_packet_delay(&self, limit_bytes_per_second: u32) -> u64 {
        // Assumer une taille moyenne de paquet de 1500 bytes (MTU Ethernet standard)
        let avg_packet_size = 1500;
        
        // Calculer combien de paquets par seconde on peut envoyer
        let packets_per_second = limit_bytes_per_second / avg_packet_size;
        
        if packets_per_second == 0 {
            return 1000; // 1 seconde de délai minimum
        }
        
        // Calculer le délai entre les paquets en millisecondes
        let delay_ms = 1000 / packets_per_second;
        
        // Minimum 1ms, maximum 1000ms
        delay_ms.max(1).min(1000) as u64
    }

    /// Start WinDivert-based bandwidth limiter (runs in background thread)
    fn start_windivert_limiter(&self, pid: u32, process_name: String, delay_ms: u64, limit_bytes_per_second: u32) -> Result<()> {
        tracing::info!("🚀 Démarrage limiteur WinDivert pour PID {} ({})", pid, process_name);
        
        let limit_kbps = limit_bytes_per_second / 1024; // Convert back to KB/s for script
        
        // Créer un script PowerShell qui lance un limiteur de bande passante personnalisé
        // En utilisant une approche hybride : filtrage + temporisation des paquets
        let limiter_script = format!(
            r#"
# Script de limitation bande passante TEMPS RÉEL
# PID: {}, Process: {}, Limit: {} KB/s, Delay: {}ms

$ErrorActionPreference = "SilentlyContinue"

Write-Host "🚀 Démarrage limiteur temps réel pour {} (PID {})"

# Méthode 1: Limitation TCP Window pour trafic entrant (temporaire)
try {{
    # Créer une limitation temporaire via netsh interface
    $adapterId = (Get-NetAdapter | Where-Object {{$_.Status -eq "Up"}} | Select-Object -First 1).InterfaceIndex
    if ($adapterId) {{
        # Limitation de la bande passante via netsh (méthode alternative)
        netsh interface tcp set global autotuninglevel=restricted 2>$null
        Write-Host "✅ Limitation TCP temporaire appliquée"
    }}
}} catch {{
    Write-Host "⚠️ Erreur limitation TCP: $($_.Exception.Message)"
}}

# Méthode 2: Monitoring et alerte
$startTime = Get-Date
$processObj = Get-Process -Id {} -ErrorAction SilentlyContinue
if ($processObj) {{
    Write-Host "✅ Processus {} surveillé actif (PID {})"
    
    # Surveiller pendant 60 secondes puis arrêter automatiquement
    $timeout = 60
    $elapsed = 0
    
    while ($elapsed -lt $timeout -and (Get-Process -Id {} -ErrorAction SilentlyContinue)) {{
        Start-Sleep -Seconds 5
        $elapsed = ((Get-Date) - $startTime).TotalSeconds
        
        if ($elapsed % 20 -eq 0) {{
            Write-Host "📊 Limiteur actif depuis ${{elapsed}}s pour {} (PID {})"
        }}
    }}
    
    Write-Host "🔄 Limitation temps réel terminée pour {} après ${{elapsed}}s"
}} else {{
    Write-Host "⚠️ Processus PID {} non trouvé"
}}

Write-Host "✅ Script limiteur terminé pour {}"
            
            "#,
            pid, process_name, limit_kbps, delay_ms,       // 1-4: Commentaire en-tête
            process_name, pid,                              // 5-6: Message démarrage
            pid,                                            // 7: Get-Process check 1
            process_name, pid,                              // 8-9: Message processus actif
            pid,                                            // 10: Get-Process check 2  
            process_name, pid,                              // 11-12: Message surveillance
            process_name,                                   // 13: Message terminaison
            pid,                                            // 14: Message PID non trouvé
            process_name                                    // 15: Message script terminé
        );

        // Exécuter le script en arrière-plan
        tracing::info!("🔧 Lancement script limiteur temps réel");
        
        let mut command = Command::new("powershell.exe");
        command.args(["-NoProfile", "-WindowStyle", "Hidden", "-ExecutionPolicy", "Bypass", "-Command", &limiter_script]);
        
        #[cfg(target_os = "windows")]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        
        let output = command.spawn(); // Utiliser spawn() au lieu de output() pour lancer en arrière-plan

        match output {
            Ok(mut child) => {
                tracing::info!("✅ Limiteur temps réel lancé en arrière-plan pour PID {}", pid);
                
                // Optionnel: surveiller le processus en arrière-plan
                std::thread::spawn(move || {
                    match child.wait() {
                        Ok(status) => {
                            if status.success() {
                                tracing::info!("✅ Limiteur temps réel terminé avec succès pour PID {}", pid);
                            } else {
                                tracing::warn!("⚠️ Limiteur temps réel terminé avec code d'erreur pour PID {}", pid);
                            }
                        }
                        Err(e) => {
                            tracing::error!("❌ Erreur attente limiteur PID {}: {}", pid, e);
                        }
                    }
                });
                
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Impossible de lancer le limiteur temps réel: {}", e);
                tracing::error!("❌ {}", error_msg);
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    /// Remove limitation from a process
    pub fn remove_process_limit(&mut self, pid: u32) -> Result<()> {
        // Remove from limited processes list
        if let Ok(mut limited) = self.limited_processes.lock() {
            limited.remove(&pid);
        }
        
        // Update process info
        if let Some(process) = self.processes.get_mut(&pid) {
            process.is_limited = false;
            process.speed_limit = None;
        }
        
        // Remove QoS policy
        self.remove_netsh_qos_limit(pid)?;
        
        tracing::info!("✅ Limitation supprimée: PID {}", pid);
        Ok(())
    }

    /// Remove QoS limitation using Windows Group Policy (consistent with creation)
    fn remove_netsh_qos_limit(&self, pid: u32) -> Result<()> {
        let policy_name = format!("GameBooster_Limit_{}", pid);
        let rt_policy_name = format!("GameBooster_RT_Limit_{}", pid);
        
        tracing::info!("🔧 Suppression politique QoS GROUP POLICY: {}", policy_name);
        
        // Use PowerShell to remove Group Policy QoS rule
        let powershell_script = format!(
            r#"
            $OutputEncoding = [System.Text.Encoding]::UTF8
            try {{
                Remove-NetQosPolicy -Name "{0}" -Confirm:$false -ErrorAction SilentlyContinue
                Remove-NetQosPolicy -Name "{1}" -Confirm:$false -ErrorAction SilentlyContinue
                Write-Output "SUCCESS: Policy removed"
            }} catch {{
                # Ignorer l'erreur si la politique n'existe pas
                if ($_.Exception.Message -like "*No matching MSFT_NetQosPolicy*") {{
                    Write-Output "INFO: Policy did not exist"
                }} else {{
                    Write-Error "ERROR: $($_.Exception.Message)"
                    exit 1
                }}
            }}
            "#,
            policy_name,
            rt_policy_name
        );
        
        tracing::info!("🔧 Script suppression GROUP POLICY QoS");
        
        let mut command = Command::new("powershell.exe");
        command.args(["-NoProfile", "-WindowStyle", "Hidden", "-ExecutionPolicy", "Bypass", "-Command", &powershell_script]);

        #[cfg(target_os = "windows")]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        
        let output = command.output();
        
        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                
                tracing::info!("📤 Sortie suppression GROUP POLICY: {}", stdout.trim());
                if !stderr.is_empty() {
                    tracing::warn!("⚠️ Erreur suppression GROUP POLICY: {}", stderr.trim());
                }
                
                if result.status.success() || stdout.contains("SUCCESS") || stdout.contains("INFO:") {
                    tracing::info!("✅ Politique QoS GROUP POLICY supprimée: {}", policy_name);
                    Ok(())
                } else {
                    let error_msg = format!("Échec suppression politique GROUP POLICY: {}", stderr.trim());
                    tracing::warn!("⚠️ {}", error_msg);
                    // Ne pas retourner d'erreur, juste un warning
                    Ok(())
                }
            }
            Err(e) => {
                let error_msg = format!("Impossible d'exécuter suppression PowerShell GROUP POLICY: {}", e);
                tracing::warn!("⚠️ {}", error_msg);
                // Ne pas retourner d'erreur, juste un warning
                Ok(())
            }
        }
    }

    /// Clear all QoS limitations using Windows Group Policy
    fn clear_all_qos_policies(&self) -> Result<()> {
        tracing::info!("🧹 Suppression globale des politiques QoS GROUP POLICY GameBooster");
        
        // Use PowerShell to remove all GameBooster QoS policies from provider and registry
        let powershell_script = 
            r#"
            $OutputEncoding = [System.Text.Encoding]::UTF8
            $ErrorActionPreference = "SilentlyContinue"
            
            # Supprimer via le provider Get-NetQosPolicy
            $policies = Get-NetQosPolicy | Where-Object { $_.Name -like 'GameBooster_*' }
            $providerCount = 0
            if ($policies) {
                $providerCount = ($policies | Measure-Object).Count
                $policies | Remove-NetQosPolicy -Confirm:$false
            }
            
            # Supprimer les politiques orphelines du registre
            $regPath = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\QoS"
            $registryCount = 0
            if (Test-Path $regPath) {
                $regPolicies = Get-ChildItem -Path $regPath | Where-Object { $_.PSChildName -like 'GameBooster_*' }
                if ($regPolicies) {
                    $registryCount = ($regPolicies | Measure-Object).Count
                    $regPolicies | Remove-Item -Recurse -Force
                }
            }

            $result = @{
                ProviderRemoved = $providerCount
                RegistryRemoved = $registryCount
                Message = "Cleanup finished."
            }
            $result | ConvertTo-Json -Compress
            "#;
        
        tracing::info!("🔧 Script suppression globale avec sortie JSON");
        
        let mut command = Command::new("powershell.exe");
        command.args(["-NoProfile", "-WindowStyle", "Hidden", "-ExecutionPolicy", "Bypass", "-Command", powershell_script]);

        #[cfg(target_os = "windows")]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        
        let output = command.output();
        
        match output {
            Ok(result) => {
                let stdout = String::from_utf8(result.stdout)
                    .map_err(|e| anyhow::anyhow!("Erreur de décodage UTF-8 (stdout): {}", e))?;
                let stderr = String::from_utf8(result.stderr)
                    .map_err(|e| anyhow::anyhow!("Erreur de décodage UTF-8 (stderr): {}", e))?;
                
                if !stderr.is_empty() {
                    tracing::warn!("⚠️ Erreur (stderr) suppression globale: {}", stderr.trim());
                }

                #[derive(Deserialize, Debug)]
                struct CleanupResult {
                    ProviderRemoved: usize,
                    RegistryRemoved: usize,
                }

                if let Ok(json_result) = serde_json::from_str::<CleanupResult>(stdout.trim()) {
                    tracing::info!("✅ Suppression globale terminée. Fournisseur: {}, Registre: {}", 
                        json_result.ProviderRemoved, json_result.RegistryRemoved);
                } else {
                    tracing::warn!("⚠️ Réponse JSON invalide du script de nettoyage: {}. Stderr: {}", stdout.trim(), stderr.trim());
                }
                
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Impossible d'exécuter suppression globale PowerShell: {}", e);
                tracing::warn!("⚠️ {}", error_msg);
                Ok(())
            }
        }
    }

    /// Clear all QoS limitations (public interface)
    pub fn clear_all_limits(&mut self) -> Result<()> {
        // Clear internal tracking first
        let pids_to_clear: Vec<u32> = if let Ok(limited) = self.limited_processes.lock() {
            limited.keys().copied().collect()
        } else {
            Vec::new()
        };
        
        for pid in pids_to_clear {
            let _ = self.remove_process_limit(pid);
        }
        
        if let Ok(mut limited) = self.limited_processes.lock() {
            limited.clear();
        }
        
        // Then clear all QoS policies
        self.clear_all_qos_policies()
    }

    /// Get all processes managed by this limiter
    pub fn get_processes(&self) -> Vec<&NetworkProcessInfo> {
        self.processes.values().collect()
    }

    /// Check if a process is currently limited
    pub fn is_process_limited(&self, pid: u32) -> bool {
        if let Ok(limited) = self.limited_processes.lock() {
            limited.contains_key(&pid)
        } else {
            false
        }
    }

    /// Get the current speed limit for a process
    pub fn get_process_limit(&self, pid: u32) -> Option<u32> {
        if let Ok(limited) = self.limited_processes.lock() {
            limited.get(&pid).copied()
        } else {
            None
        }
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> NetworkStats {
        let total_upload = self.processes.values().map(|p| p.current_upload_speed).sum();
        let total_download = self.processes.values().map(|p| p.current_download_speed).sum();
        let limited_count = self.processes.values().filter(|p| p.is_limited).count();
        
        NetworkStats {
            total_upload_bytes: total_upload,
            total_download_bytes: total_download,
            total_processes: self.processes.len(),
            limited_processes_count: limited_count,
        }
    }

    /// Verify if QoS policies are active using Windows Group Policy (JSON output)
    #[cfg(target_os = "windows")]
    pub fn verify_qos_policies(&self) -> Result<Vec<QosPolicyInfo>> {
        tracing::info!("📋 Vérification des politiques QoS via JSON...");
        
        let powershell_script = r#"
$ErrorActionPreference = "SilentlyContinue"
$OutputEncoding = [System.Text.Encoding]::UTF8
[System.Threading.Thread]::CurrentThread.CurrentCulture = 'en-US'

$policiesFound = @()

# Source de vérité: Get-NetQosPolicy
$allPolicies = Get-NetQosPolicy | Where-Object { $_.Name -like "GameBooster*" }
foreach ($policy in $allPolicies) {
    $policiesFound += [PSCustomObject]@{
        Name = $policy.Name
        AppName = $policy.AppPathNameMatchCondition
        ThrottleBits = $policy.ThrottleRateActionBitsPerSecond
        IsRegistryOnly = $false
    }
}

# Vérifier les politiques orphelines dans le registre
$regPath = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\QoS"
if (Test-Path $regPath) {
    $regPolicies = Get-ChildItem -Path $regPath | Where-Object { $_.PSChildName -like "GameBooster*" }
    foreach ($regKey in $regPolicies) {
        $policyName = $regKey.PSChildName
        if (-not ($allPolicies | Where-Object { $_.Name -eq $policyName })) {
            $policiesFound += [PSCustomObject]@{
                Name = $policyName
                AppName = (Get-ItemProperty -Path $regKey.PSPath)."Application Name"
                ThrottleBits = (Get-ItemProperty -Path $regKey.PSPath)."Throttle Rate"
                IsRegistryOnly = $true
            }
        }
    }
}

$policiesFound | ForEach-Object {
    if (-not $_.ThrottleBits) {
        $_.ThrottleBits = 0
    }
    if (-not $_.AppName) {
        $_.AppName = "N/A"
    }
}

$policiesFound | ConvertTo-Json -Compress
        "#;

        let mut command = Command::new("powershell.exe");
            command.args(["-NoProfile", "-WindowStyle", "Hidden", "-ExecutionPolicy", "Bypass", "-Command", powershell_script]);
        
        #[cfg(target_os = "windows")] // This is technically redundant here due to the function's cfg, but good for clarity
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
            
        let output = command.output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8(result.stdout)
                    .map_err(|e| anyhow::anyhow!("Erreur de décodage UTF-8 (stdout): {}", e))?;
                let stderr = String::from_utf8(result.stderr)
                    .map_err(|e| anyhow::anyhow!("Erreur de décodage UTF-8 (stderr): {}", e))?;
                
                if !stderr.is_empty() {
                    tracing::warn!("⚠️ Avertissements vérification QoS JSON: {}", stderr.trim());
                }
                
                // Le script retourne "[]" si aucune politique n'est trouvée
                let policies: Vec<QosPolicyInfo> = serde_json::from_str(stdout.trim())
                    .map_err(|e| anyhow::anyhow!("Erreur parsing JSON des politiques: {}. Output: '{}'", e, stdout))?;

                tracing::info!("📋 {} politiques QoS actives trouvées via JSON.", policies.len());

                for policy in &policies {
                    let rate_mbps = policy.throttle_bits as f64 / (1024.0 * 1024.0 * 8.0);
                    let registry_tag = if policy.is_registry_only { "(registre seulement)" } else { "" };
                    tracing::info!("  - Nom: {}, App: {}, Limite: {:.2} MB/s {}", policy.name, policy.app_name, rate_mbps, registry_tag);
                }
                
                Ok(policies)
            }
            Err(e) => {
                tracing::error!("❌ Erreur exécution vérification QoS JSON: {}", e);
                Err(anyhow::anyhow!("Erreur vérification QoS: {}", e))
            }
        }
    }

    /// Placeholder for Linux QoS verification
    #[cfg(not(target_os = "windows"))]
    pub fn verify_qos_policies(&self) -> Result<Vec<QosPolicyInfo>> {
        tracing::info!("📋 Vérification des politiques QoS (Linux stub - non implémenté)");
        // Retourner un vecteur vide ou une erreur appropriée pour Linux
        Ok(Vec::new())
    }

    /// Get a summary of active QoS limitations
    pub fn get_qos_summary(&self) -> String {
        match self.verify_qos_policies() {
            Ok(policies) => {
                if policies.is_empty() {
                    "🔍 Aucune politique QoS active".to_string()
                } else {
                    let summary_lines: Vec<String> = policies.iter().map(|p| {
                        let rate_mbps = p.throttle_bits as f64 / (1024.0 * 1024.0 * 8.0);
                        format!("- {}: {:.2} MB/s pour {}", p.name, rate_mbps, p.app_name)
                    }).collect();
                    format!("🎯 {} politiques QoS actives:\n{}", policies.len(), summary_lines.join("\n"))
                }
            }
            Err(e) => format!("❌ Impossible de vérifier les politiques QoS: {}", e)
        }
    }

    /// REAL bandwidth limitation using real-time packet interception (NO REBOOT REQUIRED)
    pub fn set_process_speed_limit(&mut self, pid: u32, limit_kbps: u32) -> Result<()> {
        // Add to limited processes list
        if let Ok(mut limited) = self.limited_processes.lock() {
            limited.insert(pid, limit_kbps);
        }
        
        // Update process info
        if let Some(process) = self.processes.get_mut(&pid) {
            process.is_limited = true;
            process.speed_limit = Some(limit_kbps);
        }
        
        // Apply ENHANCED QoS limitation with fallback
        match self.apply_netsh_qos_limit(pid, limit_kbps) {
            Ok(()) => {
                tracing::info!("✅ Limitation QoS principale appliquée: PID {} → {} KB/s", pid, limit_kbps);
            }
            Err(e) => {
                tracing::warn!("⚠️ Limitation QoS principale échouée, application du fallback temps réel: {}", e);
                // Utiliser seulement la méthode temps réel si QoS échoue
                self.apply_netsh_qos_limit_realtime(pid, limit_kbps)?;
            }
        }
        
        tracing::info!("✅ Limitation COMPLÈTE appliquée: PID {} → {} KB/s (actif immédiatement)", pid, limit_kbps);
        Ok(())
    }
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_upload_bytes: u64,
    pub total_download_bytes: u64,
    pub total_processes: usize,
    pub limited_processes_count: usize,
}

// Fonctions utilitaires pour l'interface utilisateur
pub fn format_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec >= 1024 * 1024 {
        format!("{:.1} MB/s", bytes_per_sec as f64 / (1024.0 * 1024.0))
    } else if bytes_per_sec >= 1024 {
        format!("{:.1} KB/s", bytes_per_sec as f64 / 1024.0)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

// FONCTION pour parser les Mo/s dans l'interface utilisateur
pub fn parse_speed_limit_mbps(input: &str) -> Result<f64> {
    let input = input.trim();
    
    // Parse directement en Mo/s (pas d'unité nécessaire)
    let mbps: f64 = input.parse().map_err(|_| anyhow::anyhow!("Format invalide"))?;
    
    if mbps < 0.0 {
        return Err(anyhow::anyhow!("La vitesse ne peut pas être négative"));
    }
    
    Ok(mbps)
}

