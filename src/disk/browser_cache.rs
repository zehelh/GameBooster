// Browser cache cleaning

use anyhow::Result;
use std::fs;
use std::path::Path;
use tokio::task;
use walkdir::WalkDir;

pub async fn clean_browser_cache() -> Result<u64> {
    let mut total_cleaned = 0u64;
    
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        // Chrome cache
        let chrome_cache = format!("{}\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache", user_profile);
        if Path::new(&chrome_cache).exists() {
            total_cleaned += clean_directory(&Path::new(&chrome_cache)).await?;
        }

        // Firefox cache
        let firefox_cache = format!("{}\\AppData\\Local\\Mozilla\\Firefox\\Profiles", user_profile);
        if Path::new(&firefox_cache).exists() {
            total_cleaned += clean_firefox_profiles(&Path::new(&firefox_cache)).await?;
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

async fn clean_firefox_profiles(profiles_dir: &Path) -> Result<u64> {
    let mut total_cleaned = 0u64;
    
    for entry in fs::read_dir(profiles_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let cache_dir = entry.path().join("cache2");
            if cache_dir.exists() {
                total_cleaned += clean_directory(&cache_dir).await?;
            }
        }
    }
    
    Ok(total_cleaned)
}

pub fn get_browser_cache_size() -> Result<u64> {
    let mut total_size = 0u64;
    
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let cache_dirs = vec![
            format!("{}\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache", user_profile),
            format!("{}\\AppData\\Local\\Microsoft\\Edge\\User Data\\Default\\Cache", user_profile),
            format!("{}\\AppData\\Local\\Opera Software\\Opera Stable\\Cache", user_profile),
        ];

        for cache_dir in cache_dirs {
            if Path::new(&cache_dir).exists() {
                total_size += calculate_directory_size(&Path::new(&cache_dir))?;
            }
        }

        // Firefox profiles
        let firefox_profiles = format!("{}\\AppData\\Local\\Mozilla\\Firefox\\Profiles", user_profile);
        if Path::new(&firefox_profiles).exists() {
            total_size += calculate_firefox_cache_size(&Path::new(&firefox_profiles))?;
        }
    }

    Ok(total_size)
}

pub async fn get_browser_cache_size_async() -> Result<u64> {
    task::spawn_blocking(move || {
        get_browser_cache_size()
    }).await?
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

fn calculate_firefox_cache_size(profiles_dir: &Path) -> Result<u64> {
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

pub fn clean_browser_cache_sync() -> Result<u64> {
    let mut total_cleaned = 0u64;
    
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        // Chrome cache
        let chrome_cache = format!("{}\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache", user_profile);
        if Path::new(&chrome_cache).exists() {
            total_cleaned += clean_directory_sync(&Path::new(&chrome_cache))?;
        }

        // Firefox cache
        let firefox_cache = format!("{}\\AppData\\Local\\Mozilla\\Firefox\\Profiles", user_profile);
        if Path::new(&firefox_cache).exists() {
            total_cleaned += clean_firefox_profiles_sync(&Path::new(&firefox_cache))?;
        }

        // Edge cache
        let edge_cache = format!("{}\\AppData\\Local\\Microsoft\\Edge\\User Data\\Default\\Cache", user_profile);
        if Path::new(&edge_cache).exists() {
            total_cleaned += clean_directory_sync(&Path::new(&edge_cache))?;
        }

        // Opera cache
        let opera_cache = format!("{}\\AppData\\Local\\Opera Software\\Opera Stable\\Cache", user_profile);
        if Path::new(&opera_cache).exists() {
            total_cleaned += clean_directory_sync(&Path::new(&opera_cache))?;
        }
    }

    Ok(total_cleaned)
}

fn clean_directory_sync(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                let file_size = metadata.len();
                
                // Try to delete the file
                if fs::remove_file(entry.path()).is_ok() {
                    total_size += file_size;
                    tracing::debug!("Deleted browser cache file: {}", entry.path().display());
                } else {
                    tracing::warn!("Failed to delete browser cache file: {}", entry.path().display());
                }
            }
        }
    }

    Ok(total_size)
}

fn clean_firefox_profiles_sync(profiles_dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    
    for entry in WalkDir::new(profiles_dir).max_depth(2).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() && entry.file_name().to_string_lossy().contains("cache") {
            total_size += clean_directory_sync(entry.path())?;
        }
    }

    Ok(total_size)
}
