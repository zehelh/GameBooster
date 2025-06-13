// Browser cache cleaning

use anyhow::Result;
use std::fs;
use std::path::{Path};
use walkdir::WalkDir;

pub async fn clean_browser_cache() -> Result<u64> {
    let mut total_cleaned = 0u64;

    #[cfg(target_os = "windows")]
    {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            // Chrome cache
            let chrome_cache = format!("{}\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache", user_profile);
            if Path::new(&chrome_cache).exists() {
                total_cleaned += clean_directory(&Path::new(&chrome_cache)).await?;
            }

            // Firefox cache
            let firefox_cache_base = format!("{}\\AppData\\Local\\Mozilla\\Firefox\\Profiles", user_profile);
            if Path::new(&firefox_cache_base).exists() {
                total_cleaned += clean_firefox_profiles_windows(&Path::new(&firefox_cache_base)).await?;
            }

            // Edge cache
            let edge_cache = format!("{}\\AppData\\Local\\Microsoft\\Edge\\User Data\\Default\\Cache", user_profile);
            if Path::new(&edge_cache).exists() {
                total_cleaned += clean_directory(&Path::new(&edge_cache)).await?;
            }

            // Opera cache
            let opera_cache = format!("{}\\AppData\\Local\\Opera Software\\Opera Stable\\Cache", user_profile);
            if Path::new(&opera_cache).exists() {
                total_cleaned += clean_directory(&Path::new(&opera_cache)).await?;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home_dir) = dirs::home_dir() {
            // Chrome cache
            let chrome_cache = home_dir.join(".cache/google-chrome/Default/Cache");
            if chrome_cache.exists() {
                total_cleaned += clean_directory(&chrome_cache).await?;
            }
            // Chromium cache
            let chromium_cache = home_dir.join(".cache/chromium/Default/Cache");
            if chromium_cache.exists() {
                total_cleaned += clean_directory(&chromium_cache).await?;
            }

            // Firefox cache
            let firefox_cache_base = home_dir.join(".mozilla/firefox");
            if firefox_cache_base.exists() {
                total_cleaned += clean_firefox_profiles_linux(&firefox_cache_base).await?;
            }
             // Edge cache (snap)
            let edge_snap_cache = home_dir.join("snap/microsoft-edge-dev/current/.cache/microsoft-edge-dev/Default/Cache");
             if edge_snap_cache.exists() {
                total_cleaned += clean_directory(&edge_snap_cache).await?;
            }
            // Edge cache (flatpak)
            let edge_flatpak_cache = home_dir.join(".var/app/com.microsoft.Edge/cache/Microsoft/Edge/Default/Cache");
            if edge_flatpak_cache.exists() {
                total_cleaned += clean_directory(&edge_flatpak_cache).await?;
            }
        }
    }

    Ok(total_cleaned)
}

async fn clean_directory(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                let file_size = metadata.len();
                
                // Try to delete the file
                if fs::remove_file(entry.path()).is_ok() {
                    total_size += file_size;
                }
            }
        }
    }
    
    Ok(total_size)
}

#[cfg(target_os = "windows")]
async fn clean_firefox_profiles_windows(profiles_dir: &Path) -> Result<u64> {
    let mut total_cleaned = 0u64;
    for entry in fs::read_dir(profiles_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let cache_dir = entry.path().join("cache2"); // Windows specific sub-path
            if cache_dir.exists() {
                total_cleaned += clean_directory(&cache_dir).await?;
            }
        }
    }
    Ok(total_cleaned)
}

#[cfg(target_os = "linux")]
async fn clean_firefox_profiles_linux(profiles_dir: &Path) -> Result<u64> {
    let mut total_cleaned = 0u64;
    for entry in fs::read_dir(profiles_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && entry.file_name().to_string_lossy().ends_with(".default-release") {
             // Common pattern for default profile, cache might be directly inside or in a subfolder
            let cache_dir_variant1 = path.join("cache2"); // Check for cache2
            if cache_dir_variant1.exists() {
                total_cleaned += clean_directory(&cache_dir_variant1).await?;
            }
            let cache_dir_variant2 = path.join("startupCache"); // Check for startupCache (less common for bulk data)
             if cache_dir_variant2.exists() {
                total_cleaned += clean_directory(&cache_dir_variant2).await?;
            }
        }
    }
    Ok(total_cleaned)
}


pub fn get_browser_cache_size() -> Result<u64> {
    let mut total_size = 0u64;

    #[cfg(target_os = "windows")]
    {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let cache_dirs = vec![
                format!("{}\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache", user_profile),
                format!("{}\\AppData\\Local\\Microsoft\\Edge\\User Data\\Default\\Cache", user_profile),
                format!("{}\\AppData\\Local\\Opera Software\\Opera Stable\\Cache", user_profile),
            ];

            for cache_dir_str in cache_dirs {
                let cache_dir = Path::new(&cache_dir_str);
                if cache_dir.exists() {
                    total_size += calculate_directory_size(&cache_dir)?;
                }
            }

            // Firefox profiles
            let firefox_profiles = format!("{}\\AppData\\Local\\Mozilla\\Firefox\\Profiles", user_profile);
            if Path::new(&firefox_profiles).exists() {
                total_size += calculate_firefox_cache_size_windows(&Path::new(&firefox_profiles))?;
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Some(home_dir) = dirs::home_dir() {
            let cache_paths = vec![
                home_dir.join(".cache/google-chrome/Default/Cache"),
                home_dir.join(".cache/chromium/Default/Cache"),
                home_dir.join("snap/microsoft-edge-dev/current/.cache/microsoft-edge-dev/Default/Cache"),
                home_dir.join(".var/app/com.microsoft.Edge/cache/Microsoft/Edge/Default/Cache"),
            ];
            for path in cache_paths {
                if path.exists() {
                    total_size += calculate_directory_size(&path)?;
                }
            }
            let firefox_base = home_dir.join(".mozilla/firefox");
            if firefox_base.exists() {
                total_size += calculate_firefox_cache_size_linux(&firefox_base)?;
            }
        }
    }

    Ok(total_size)
}

fn calculate_directory_size(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }
    
    Ok(total_size)
}

#[cfg(target_os = "windows")]
fn calculate_firefox_cache_size_windows(profiles_dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    for entry in fs::read_dir(profiles_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let cache_dir = entry.path().join("cache2");
            if cache_dir.exists() {
                total_size += calculate_directory_size(&cache_dir)?;
            }
        }
    }
    Ok(total_size)
}

#[cfg(target_os = "linux")]
fn calculate_firefox_cache_size_linux(profiles_dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
     for entry in fs::read_dir(profiles_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && entry.file_name().to_string_lossy().ends_with(".default-release") {
            let cache_dir_variant1 = path.join("cache2");
            if cache_dir_variant1.exists() {
                total_size += calculate_directory_size(&cache_dir_variant1)?;
            }
            let cache_dir_variant2 = path.join("startupCache");
             if cache_dir_variant2.exists() {
                total_size += calculate_directory_size(&cache_dir_variant2)?;
            }
        }
    }
    Ok(total_size)
}
