use anyhow::{Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use windows_sys::Win32::Foundation::{CloseHandle, BOOL, MAX_PATH};
use windows_sys::Win32::System::ProcessStatus::{
    EmptyWorkingSet, EnumProcesses, GetModuleBaseNameW, K32GetProcessMemoryInfo,
    PROCESS_MEMORY_COUNTERS,
};
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
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

// Fonction pour obtenir les informations sur la mémoire système
pub fn get_system_memory_info() -> (u64, u64) {
    let mut mem_info: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    mem_info.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut mem_info) } != 0 {
        (mem_info.ullTotalPhys, mem_info.ullTotalPhys - mem_info.ullAvailPhys)
    } else {
        (0, 0)
    }
}

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