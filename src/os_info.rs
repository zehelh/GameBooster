// This module provides functions to get OS-specific information.

// We will need to implement a new way to get the version using windows_sys
// For now, let's return a placeholder.

use winver::WindowsVersion;

pub fn get_windows_version_string() -> String {
    if let Some(version) = WindowsVersion::detect() {
        format!("{}", version)
    } else {
        "Version de Windows inconnue".to_string()
    }
}