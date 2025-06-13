//! # Utilities
//!
//! A module for shared utility functions.

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
#[cfg(target_os = "windows")]
use std::ffi::c_void;

/// Checks if the current process is elevated (running as administrator or root).
pub fn is_elevated() -> bool {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let mut token: HANDLE = std::ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return false;
            }

            let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
            let mut return_length: u32 = 0;
            
            let result = GetTokenInformation(
                token,
                TokenElevation,
                &mut elevation as *mut TOKEN_ELEVATION as *mut c_void,
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            );

            CloseHandle(token);

            result != 0 && elevation.TokenIsElevated != 0
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // On Linux, check if UID is 0 (root)
        unsafe { libc::getuid() == 0 }
    }
}

/// Checks if a process name corresponds to a common Windows system process.
/// This helps in filtering out critical processes from user-facing lists.
pub fn is_windows_system_process(process_name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
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
    #[cfg(not(target_os = "windows"))]
    {
        // Sur Linux, la notion de "processus système" à ne pas toucher est différente.
        // On pourrait filtrer par UID (root), ou des noms communs comme "systemd", "kthreadd", etc.
        // Pour l'instant, on retourne false pour ne pas filtrer agressivement.
        let _ = process_name; // Évite l'avertissement unused_variables
        false
    }
}