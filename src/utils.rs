//! # Utilities
//!
//! A module for shared utility functions.

/// Checks if a process name corresponds to a common Windows system process.
/// This helps in filtering out critical processes from user-facing lists.
pub fn is_windows_system_process(process_name: &str) -> bool {
    let lower_process_name = process_name.to_lowercase();
    let windows_processes = [
        "svchost.exe", "system", "registry", "dwm.exe", "winlogon.exe",
        "csrss.exe", "lsass.exe", "services.exe", "spoolsv.exe",
        "explorer.exe", "taskhost.exe", "rundll32.exe", "dllhost.exe",
        "msiexec.exe", "conhost.exe", "audiodg.exe", "wininit.exe",
        "fontdrvhost.exe", "sihost.exe", "ctfmon.exe"
    ];

    windows_processes.iter().any(|&sys_proc| 
        lower_process_name.contains(sys_proc)
    )
} 