use anyhow::Result;
use chrono::Local;
use windows::{
    Win32::{
        Foundation::{CloseHandle, HMODULE, INVALID_HANDLE_VALUE, MAX_PATH},
        System::{
            ProcessStatus::{EnumProcesses, GetProcessMemoryInfo, GetModuleBaseNameW, EmptyWorkingSet},
            Threading::{GetCurrentProcess, OpenProcess, PROCESS_ALL_ACCESS},
        },
    },
};

// Structure pour stocker les informations d'un processus nettoyé
#[derive(Clone)]
pub struct CleanedProcess {
    pub name: String,
    pub memory_freed: usize,
}

// Structure pour stocker les résultats du nettoyage
#[derive(Clone)]
pub struct CleaningResults {
    pub processes: Vec<CleanedProcess>,
    pub cleaned_count: usize,
    pub total_memory_before: usize,
    pub total_memory_after: usize,
    pub global_clean_success: bool,
    pub start_time: chrono::DateTime<Local>,
    pub end_time: Option<chrono::DateTime<Local>>,
    pub is_completed: bool,
    pub has_error: bool,
    pub error_message: String,
}

impl CleaningResults {
    pub fn new() -> Self {
        CleaningResults {
            processes: Vec::new(),
            cleaned_count: 0,
            total_memory_before: 0,
            total_memory_after: 0,
            global_clean_success: false,
            start_time: Local::now(),
            end_time: None,
            is_completed: false,
            has_error: false,
            error_message: String::new(),
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

#[repr(C)]
struct PROCESS_MEMORY_COUNTERS {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    page_file_usage: usize,
    peak_page_file_usage: usize,
}

// Fonction principale pour nettoyer la mémoire
pub fn clean_memory() -> Result<CleaningResults, String> {
    let mut results = CleaningResults::new();

    // Obtenir les processus
    let mut processes = Vec::with_capacity(1024);
    processes.resize(1024, 0);
    let mut bytes_needed = 0;

    let process_enum_result = unsafe {
        EnumProcesses(
            processes.as_mut_ptr(),
            (processes.len() * std::mem::size_of::<u32>()) as u32,
            &mut bytes_needed,
        )
    };

    if process_enum_result.is_err() {
        return Err("Échec de l'énumération des processus".to_string());
    }

    let process_count = bytes_needed as usize / std::mem::size_of::<u32>();
    let processes = &processes[0..process_count];

    // Libération globale de la mémoire système
    let current_process = unsafe { GetCurrentProcess() };
    
    // Utiliser EmptyWorkingSet pour nettoyer le processus actuel
    let global_clean_result = unsafe { EmptyWorkingSet(current_process) };
    results.global_clean_success = global_clean_result.is_ok();

    // Pour chaque processus
    for &pid in processes {
        if pid == 0 {
            continue;
        }

        // Ouvrir un handle vers le processus avec accès complet
        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid) };

        if let Ok(handle) = handle {
            if handle != INVALID_HANDLE_VALUE {
                // Essayer d'obtenir le nom du processus
                let mut name_buffer = [0u16; MAX_PATH as usize];
                let name_len = unsafe { 
                    GetModuleBaseNameW(
                        handle, 
                        HMODULE(0), 
                        &mut name_buffer
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
                    page_fault_count: 0,
                    peak_working_set_size: 0,
                    working_set_size: 0,
                    quota_peak_paged_pool_usage: 0,
                    quota_paged_pool_usage: 0,
                    quota_peak_non_paged_pool_usage: 0,
                    quota_non_paged_pool_usage: 0,
                    page_file_usage: 0,
                    peak_page_file_usage: 0,
                };

                let memory_info_result = unsafe { 
                    GetProcessMemoryInfo(
                        handle, 
                        &mut mem_counters as *mut PROCESS_MEMORY_COUNTERS as *mut _, 
                        std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32
                    )
                };

                let before_memory = if memory_info_result.is_ok() {
                    mem_counters.working_set_size
                } else {
                    0
                };

                results.total_memory_before += before_memory;

                // Tenter le nettoyage de la mémoire du processus avec EmptyWorkingSet
                let success = unsafe { EmptyWorkingSet(handle) };

                if success.is_ok() {
                    // Mesurer à nouveau la mémoire après le nettoyage
                    let memory_info_result = unsafe { 
                        GetProcessMemoryInfo(
                            handle, 
                            &mut mem_counters as *mut PROCESS_MEMORY_COUNTERS as *mut _, 
                            std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32
                        )
                    };
                    
                    let after_memory = if memory_info_result.is_ok() {
                        mem_counters.working_set_size
                    } else {
                        0
                    };

                    results.total_memory_after += after_memory;

                    // Calculer la mémoire libérée
                    let freed_memory = if before_memory > after_memory {
                        before_memory - after_memory
                    } else {
                        0
                    };

                    if freed_memory > 0 {
                        results.cleaned_count += 1;
                        results.processes.push(CleanedProcess {
                            name: process_name,
                            memory_freed: freed_memory,
                        });
                    }
                }

                unsafe { let _ = CloseHandle(handle); }
            }
        }
    }

    results.is_completed = true;
    results.end_time = Some(Local::now());
    Ok(results)
}

// Fonction pour obtenir les informations sur la mémoire système
pub fn get_system_memory_info() -> (usize, usize) {
    // Utiliser sysinfo pour obtenir les informations sur la mémoire
    use sysinfo::System;
    
    let mut system = System::new_all();
    system.refresh_memory();
    
    // Obtenir les valeurs en Ko et les convertir en octets
    let total_memory = system.total_memory() as usize * 1024; // Ko en octets
    let available_memory = system.available_memory() as usize * 1024;
    
    println!("Mémoire système - Total: {} octets, Disponible: {} octets", 
             total_memory, available_memory);
    
    (total_memory, available_memory)
} 