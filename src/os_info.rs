// This module provides functions to get OS-specific information.

#[cfg(target_os = "windows")]
use std::mem;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::SystemInformation::{GetVersionExW, OSVERSIONINFOEXW};

#[cfg(target_os = "windows")]
pub fn get_windows_version_string() -> String {
    unsafe {
        let mut osvi: OSVERSIONINFOEXW = mem::zeroed();
        osvi.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFOEXW>() as u32;

        if GetVersionExW(&mut osvi as *mut _ as *mut _) == 0 {
            return "Windows (unknown version - GetVersionExW failed)".to_string();
        }

        let product_name = match (osvi.dwMajorVersion, osvi.dwMinorVersion) {
            (10, 0) => {
                if osvi.dwBuildNumber >= 22000 {
                    "Windows 11"
                } else {
                    "Windows 10"
                }
            }
            (6, 3) => "Windows 8.1",
            (6, 2) => "Windows 8",
            (6, 1) => "Windows 7",
            (6, 0) => "Windows Vista",
            _ => "Windows (older or unknown)",
        };

        format!(
            "{} (Build {}.{}.{})",
            product_name, osvi.dwMajorVersion, osvi.dwMinorVersion, osvi.dwBuildNumber
        )
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_windows_version_string() -> String { // Cette fonction donne maintenant une description plus détaillée pour Linux
    match sys_info::os_type() {
        Ok(os_type) => match sys_info::os_release() {
            Ok(os_release) => format!("{} ({})", os_type, os_release),
            Err(_) => os_type,
        },
        Err(_) => "Linux (version inconnue)".to_string(),
    }
}

/// Returns the OS platform as a simple string: "windows" or "linux".
pub fn get_os_platform() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else {
        "unknown".to_string() // Pour d'autres systèmes d'exploitation potentiels
    }
}

/// Returns the OS version as (major, minor, build)
#[cfg(target_os = "windows")]
pub fn get_windows_version_numbers() -> (u32, u32, u32) {
    unsafe {
        let mut osvi: OSVERSIONINFOEXW = mem::zeroed();
        osvi.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFOEXW>() as u32;
        if GetVersionExW(&mut osvi as *mut _ as *mut _) != 0 {
            (osvi.dwMajorVersion, osvi.dwMinorVersion, osvi.dwBuildNumber)
        } else {
            (0, 0, 0)
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_windows_version_numbers() -> (u32, u32, u32) {
    // Tenter de parser à partir de sys_info::os_release() peut être complexe
    // car le format n'est pas standardisé comme pour Windows.
    // Retourne (0,0,0) pour l'instant pour Linux.
    (0, 0, 0)
}