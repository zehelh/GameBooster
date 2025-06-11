// This module provides functions to get OS-specific information.

// Utilisation de windows-sys pour éviter la dépendance winver externe
use windows_sys::Win32::System::SystemInformation::GetVersion;

pub fn get_windows_version_string() -> String {
    unsafe {
        let version = GetVersion();
        let major = (version & 0xFF) as u32;
        let minor = ((version >> 8) & 0xFF) as u32;
        let build = if version < 0x80000000 { version >> 16 } else { 0 };
        
        match (major, minor) {
            (10, 0) => {
                if build >= 22000 {
                    "Windows 11".to_string()
                } else {
                    "Windows 10".to_string()
                }
            }
            (6, 3) => "Windows 8.1".to_string(),
            (6, 2) => "Windows 8".to_string(),
            (6, 1) => "Windows 7".to_string(),
            _ => format!("Windows {}.{}", major, minor),
        }
    }
}