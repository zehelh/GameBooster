// This module provides functions to get OS-specific information.

use std::mem;
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
pub fn get_windows_version_string() -> String {
    "Not running on Windows".to_string()
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
    (0, 0, 0)
}