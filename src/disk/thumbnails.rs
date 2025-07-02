// Thumbnails cleaning

use anyhow::Result;
use std::fs;
use std::path::Path;
use tokio::task;
use walkdir::WalkDir;

pub async fn clean_thumbnails() -> Result<u64> {
    let mut total_cleaned = 0u64;
    
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let thumbnails_dirs = vec![
            format!("{}\\AppData\\Local\\Microsoft\\Windows\\Explorer", user_profile),
            format!("{}\\AppData\\Local\\Packages\\Microsoft.Windows.Photos_8wekyb3d8bbwe\\LocalState\\PhotosAppCache", user_profile),
        ];

        for thumb_dir in thumbnails_dirs {
            let path = Path::new(&thumb_dir);
            if path.exists() {
                total_cleaned += clean_thumbnails_directory(path).await?;
            }
        }
    }

    Ok(total_cleaned)
}

async fn clean_thumbnails_directory(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            
            // Check if it's a thumbnail file
            if is_thumbnail_file(path) {
                if let Ok(metadata) = entry.metadata() {
                    let file_size = metadata.len();
                    
                    // Try to delete the file
                    if fs::remove_file(path).is_ok() {
                        total_size += file_size;
                    }
                }
            }
        }
    }
    
    Ok(total_size)
}

fn is_thumbnail_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            return matches!(ext_str.to_lowercase().as_str(), "db" | "thumbcache_32" | "thumbcache_96" | "thumbcache_256" | "thumbcache_1024" | "thumbcache_idx" | "thumbcache_sr");
        }
    }
    
    if let Some(filename) = path.file_name() {
        if let Some(name_str) = filename.to_str() {
            return name_str.starts_with("thumbcache_") || name_str == "Thumbs.db";
        }
    }
    
    false
}

pub fn get_thumbnails_size() -> Result<u64> {
    let mut total_size = 0u64;
    
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let thumbnails_dirs = vec![
            format!("{}/AppData/Local/Microsoft/Windows/Explorer", user_profile),
            format!("{}/AppData/Local/Packages/Microsoft.Windows.Photos_8wekyb3d8bbwe/LocalState/PhotosAppCache", user_profile),
        ];

        for thumb_dir in thumbnails_dirs {
            let path = Path::new(&thumb_dir);
            if path.exists() {
                total_size += calculate_thumbnails_size(path)?;
            }
        }
    }

    Ok(total_size)
}

pub async fn get_thumbnails_size_async() -> Result<u64> {
    task::spawn_blocking(move || {
        get_thumbnails_size()
    }).await?
}

fn calculate_thumbnails_size(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && is_thumbnail_file(entry.path()) {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }
    
    Ok(total_size)
}

pub fn clean_thumbnails_sync() -> Result<u64> {
    let mut total_cleaned = 0u64;
    
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let thumbnails_dirs = vec![
            format!("{}\\AppData\\Local\\Microsoft\\Windows\\Explorer", user_profile),
            format!("{}\\AppData\\Local\\Packages\\Microsoft.Windows.Photos_8wekyb3d8bbwe\\LocalState\\PhotosAppCache", user_profile),
        ];

        for thumb_dir in thumbnails_dirs {
            let path = Path::new(&thumb_dir);
            if path.exists() {
                total_cleaned += clean_thumbnails_directory_sync(path)?;
            }
        }
    }

    Ok(total_cleaned)
}

fn clean_thumbnails_directory_sync(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            
            // Check if it's a thumbnail file
            if is_thumbnail_file(path) {
                if let Ok(metadata) = entry.metadata() {
                    let file_size = metadata.len();
                    
                    // Try to delete the file
                    if fs::remove_file(path).is_ok() {
                        total_size += file_size;
                    }
                }
            }
        }
    }
    
    Ok(total_size)
}
