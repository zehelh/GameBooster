use anyhow::{Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sysinfo::System;
use std::process::Command;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, BOOL, MAX_PATH};
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::{
    EmptyWorkingSet, EnumProcesses, GetModuleBaseNameW, K32GetProcessMemoryInfo,
    PROCESS_MEMORY_COUNTERS,
};
#[cfg(windows)]
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA, PROCESS_VM_READ,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCleaned {
    pub name: String,
    pub memory_freed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleaningResults {
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub processes: Vec<ProcessCleaned>,
    pub total_memory_before: usize,
    pub total_memory_after: usize,
    pub has_error: bool,
    pub error_message: String,
    pub is_completed: bool,
}

impl CleaningResults {
    pub fn new() -> Self {
        CleaningResults {
            processes: Vec::new(),
            total_memory_before: 0,
            total_memory_after: 0,
            has_error: false,
            error_message: String::new(),
            is_completed: false,
            start_time: Local::now(),
            end_time: None,
        }
    }

    pub fn total_freed(&self) -> usize {
        if self.total_memory_before > self.total_memory_after {
            self.total_memory_before - self.total_memory_after
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SystemMemoryInfo {
    pub total_physical: u64,
    pub avail_physical: u64,
    pub total_pagefile: u64,
    pub avail_pagefile: u64,
}

impl SystemMemoryInfo {
    pub fn used_physical(&self) -> u64 {
        self.total_physical - self.avail_physical
    }

    pub fn used_physical_percent(&self) -> f32 {
        if self.total_physical == 0 {
            0.0
        } else {
            (self.used_physical() as f32 / self.total_physical as f32) * 100.0
        }
    }
}

// Fonction principale pour nettoyer la mémoire
#[cfg(windows)]
pub fn clean_memory() -> Result<CleaningResults> {
    let mut results = CleaningResults::new();
    let mut pids = [0u32; 2048];
    let mut bytes_returned = 0;

    if unsafe {
        EnumProcesses(
            pids.as_mut_ptr(),
            std::mem::size_of_val(&pids) as u32,
            &mut bytes_returned,
        )
    } == 0
    {
        return Err(anyhow::anyhow!("Failed to enumerate processes."));
    }

    let current_process_handle = unsafe { GetCurrentProcess() };
    unsafe { EmptyWorkingSet(current_process_handle) };

    for &pid in &pids[..bytes_returned as usize / std::mem::size_of::<u32>()] {
        if pid == 0 {
            continue;
        }

        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_SET_QUOTA,
                BOOL::from(false),
                pid,
            )
        };
        if handle != std::ptr::null_mut() {
            // Essayer d'obtenir le nom du processus
            let mut name_buffer = [0u16; MAX_PATH as usize];
            let name_len = unsafe {
                GetModuleBaseNameW(
                    handle,
                    std::ptr::null_mut(),
                    name_buffer.as_mut_ptr(),
                    MAX_PATH,
                )
            };

            let process_name = if name_len > 0 {
                String::from_utf16_lossy(&name_buffer[..name_len as usize])
            } else {
                format!("PID: {}", pid)
            };

            // Obtenir la mémoire avant le nettoyage
            let mut mem_counters = PROCESS_MEMORY_COUNTERS {
                cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                PageFaultCount: 0,
                PeakWorkingSetSize: 0,
                WorkingSetSize: 0,
                QuotaPeakPagedPoolUsage: 0,
                QuotaPagedPoolUsage: 0,
                QuotaPeakNonPagedPoolUsage: 0,
                QuotaNonPagedPoolUsage: 0,
                PagefileUsage: 0,
                PeakPagefileUsage: 0,
            };

            if unsafe {
                K32GetProcessMemoryInfo(
                    handle,
                    &mut mem_counters,
                    std::mem::size_of_val(&mem_counters) as u32,
                )
            } != 0
            {
                let before_memory = mem_counters.WorkingSetSize;
                results.total_memory_before += before_memory;

                if unsafe { EmptyWorkingSet(handle) } != 0 {
                    if unsafe {
                        K32GetProcessMemoryInfo(
                            handle,
                            &mut mem_counters,
                            std::mem::size_of_val(&mem_counters) as u32,
                        )
                    } != 0
                    {
                        let after_memory = mem_counters.WorkingSetSize;
                        results.total_memory_after += after_memory;

                        // Calculer la mémoire libérée
                        let freed_memory = if before_memory > after_memory {
                            before_memory - after_memory
                        } else {
                            0
                        };

                        if freed_memory > 0 {
                            results.processes.push(ProcessCleaned {
                                name: process_name,
                                memory_freed: freed_memory,
                            });
                        }
                    }
                }
            }

            unsafe { CloseHandle(handle) };
        }
    }

    // Sort processes by memory freed in descending order
    results.processes.sort_by(|a, b| b.memory_freed.cmp(&a.memory_freed));

    results.is_completed = true;
    results.end_time = Some(Local::now());
    Ok(results)
}

#[cfg(not(windows))]
pub fn clean_memory() -> Result<CleaningResults> {
    let mut results = CleaningResults::new();
    let mut sys = System::new_all();
    sys.refresh_memory();
    results.total_memory_before = (sys.total_memory() - sys.available_memory()) as usize;

    if utils::is_elevated() {
        // Synchroniser les données sur le disque pour éviter la perte de données
        let sync_output = Command::new("sync").output();
        if sync_output.is_err() || !sync_output.unwrap().status.success() {
            results.has_error = true;
            results.error_message = "Échec de la commande sync avant drop_caches.".to_string();
        } else {
            // Tenter de vider les caches pagecache, dentries et inodes
            // echo 1 > /proc/sys/vm/drop_caches  (PageCache)
            // echo 2 > /proc/sys/vm/drop_caches  (dentries et inodes)
            // echo 3 > /proc/sys/vm/drop_caches  (PageCache, dentries et inodes)
            let drop_caches_output = Command::new("sh")
                .arg("-c")
                .arg("echo 3 > /proc/sys/vm/drop_caches")
                .output();

            if drop_caches_output.is_err() || !drop_caches_output.unwrap().status.success() {
                results.has_error = true;
                results.error_message = "Échec de la commande drop_caches. Vérifiez les droits root.".to_string();
            } else {
                results.error_message = "Les caches système (pagecache, dentries, inodes) ont été vidés.".to_string();
            }
        }
    } else {
        results.has_error = true; // Pas une erreur bloquante, mais une info
        results.error_message = "L'application n'a pas les droits root pour vider les caches système. Cette opération est plus efficace avec les droits administrateur.".to_string();
    }

    sys.refresh_memory(); // Re-vérifier après l'opération
    results.total_memory_after = (sys.total_memory() - sys.available_memory()) as usize;
    results.is_completed = true;
    results.end_time = Some(Local::now());

    // Si une erreur s'est produite mais que de la mémoire a quand même été libérée (peu probable ici sans root)
    // ou si aucune erreur et de la mémoire libérée.
    if (!results.has_error || results.total_freed() > 0) && results.error_message.is_empty() {
        results.error_message = format!(
            "Mémoire des caches système potentiellement libérée : {} Mo",
            results.total_freed() / 1024 / 1024
        );
    } else if results.total_freed() == 0 && !results.has_error && results.error_message.is_empty() {
        results.error_message = "Aucune mémoire supplémentaire n'a pu être libérée des caches système, ou l'opération a été sautée (pas de droits root).".to_string();
    }
    // Si has_error est true, error_message est déjà rempli.

    Ok(results)
}

// Fonction pour obtenir les informations sur la mémoire système
#[cfg(windows)]
pub fn get_system_memory_info() -> (u64, u64) {
    let mut mem_info: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    mem_info.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut mem_info) } != 0 {
        (mem_info.ullTotalPhys, mem_info.ullTotalPhys - mem_info.ullAvailPhys)
    } else {
        (0, 0)
    }
}

#[cfg(not(windows))]
pub fn get_system_memory_info() -> (u64, u64) {
    // Cette fonction semble moins utilisée que get_detailed_system_memory_info
    // mais on la met à jour pour la cohérence.
    let mut sys = System::new_all();
    sys.refresh_memory();
    (sys.total_memory(), sys.total_memory() - sys.available_memory())
}

#[cfg(windows)]
pub fn get_detailed_system_memory_info() -> SystemMemoryInfo {
    let mut mem_info: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    mem_info.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut mem_info) } != 0 {
        SystemMemoryInfo {
            total_physical: mem_info.ullTotalPhys,
            avail_physical: mem_info.ullAvailPhys,
            total_pagefile: mem_info.ullTotalPageFile,
            avail_pagefile: mem_info.ullAvailPageFile,
        }
    } else {
        SystemMemoryInfo {
            total_physical: 0,
            avail_physical: 0,
            total_pagefile: 0,
            avail_pagefile: 0,
        }
    }
}

#[cfg(not(windows))]
pub fn get_detailed_system_memory_info() -> SystemMemoryInfo {
    let mut sys = System::new_all();
    sys.refresh_memory(); // Important: rafraîchir les données mémoire

    SystemMemoryInfo {
        total_physical: sys.total_memory(),
        avail_physical: sys.available_memory(),
        total_pagefile: sys.total_swap(),
        avail_pagefile: sys.free_swap(), // sys.available_swap() n'existe pas, free_swap est le plus proche
    }
}