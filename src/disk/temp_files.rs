// Temporary files cleaning

use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub async fn clean_temp_files() -> Result<u64> {
    let mut total_cleaned = 0u64;
    
    // System temp directories
    let temp_dirs = vec![
        std::env::temp_dir(),
        Path::new("C:\\Windows\\Temp").to_path_buf(),
        Path::new("C:\\Windows\\Prefetch").to_path_buf(),
    ];

    for temp_dir in temp_dirs {
        if temp_dir.exists() {
            total_cleaned += clean_directory(&temp_dir).await?;
        }
    }

    // User-specific temp directories
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let user_temp_dirs = vec![
            format!("{}\\AppData\\Local\\Temp", user_profile),
            format!("{}\\AppData\\Local\\Microsoft\\Windows\\Temporary Internet Files", user_profile),
        ];

        for temp_dir in user_temp_dirs {
            let path = Path::new(&temp_dir);
            if path.exists() {
                total_cleaned += clean_directory(path).await?;
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

pub fn get_temp_file_size() -> Result<u64> {
    let mut total_size = 0u64;
    
    let temp_dirs = vec![
        std::env::temp_dir(),
        Path::new("C:\\Windows\\Temp").to_path_buf(),
    ];

    for temp_dir in temp_dirs {
        if temp_dir.exists() {
            total_size += calculate_directory_size(&temp_dir)?;
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
