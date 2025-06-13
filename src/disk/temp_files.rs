// Temporary files cleaning

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub async fn clean_temp_files() -> Result<u64> {
    let mut total_cleaned = 0u64;

    // System temp directories
    let mut temp_dirs: Vec<PathBuf> = vec![std::env::temp_dir()];

    #[cfg(target_os = "windows")]
    {
        temp_dirs.push(Path::new("C:\\Windows\\Temp").to_path_buf());
        temp_dirs.push(Path::new("C:\\Windows\\Prefetch").to_path_buf());
    }
    #[cfg(target_os = "linux")]
    {
        temp_dirs.push(PathBuf::from("/tmp"));
        temp_dirs.push(PathBuf::from("/var/tmp"));
        // Prefetch n'a pas d'équivalent direct universel sur Linux qui soit sûr à nettoyer de cette manière.
    }

    for temp_dir in &temp_dirs {
        if temp_dir.exists() {
            total_cleaned += clean_directory(temp_dir).await?;
        }
    }

    // User-specific temp directories
    #[cfg(target_os = "windows")]
    {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let user_temp_dirs_str = vec![
                format!("{}\\AppData\\Local\\Temp", user_profile),
                format!("{}\\AppData\\Local\\Microsoft\\Windows\\Temporary Internet Files", user_profile),
            ];
            for temp_dir_str in user_temp_dirs_str {
                let path = Path::new(&temp_dir_str);
                if path.exists() {
                    total_cleaned += clean_directory(path).await?;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(home_dir) = dirs::home_dir() {
            let user_temp_dirs_path = vec![
                home_dir.join(".cache"), // Un bon candidat général pour le cache utilisateur
            ];
            for path in user_temp_dirs_path {
                if path.exists() {
                    // Nettoyer le contenu de .cache peut être agressif,
                    // il faudrait être plus sélectif ou permettre à l'utilisateur de configurer.
                    // Pour l'instant, nous allons le parcourir.
                    total_cleaned += clean_directory(&path).await?;
                }
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
    
    let mut temp_dirs_path: Vec<PathBuf> = vec![std::env::temp_dir()];
    #[cfg(target_os = "windows")]
    {
        temp_dirs_path.push(Path::new("C:\\Windows\\Temp").to_path_buf());
    }
    #[cfg(target_os = "linux")]
    {
        temp_dirs_path.push(PathBuf::from("/tmp"));
        temp_dirs_path.push(PathBuf::from("/var/tmp"));
    }


    for temp_dir in temp_dirs_path {
        if temp_dir.exists() {
            total_size += calculate_directory_size(&temp_dir)?;
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let user_temp_dirs_str = vec![
                format!("{}\\AppData\\Local\\Temp", user_profile),
            ];
            for temp_dir_str in user_temp_dirs_str {
                let path = Path::new(&temp_dir_str);
                if path.exists() {
                    total_size += calculate_directory_size(path)?;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(home_dir) = dirs::home_dir() {
            let user_temp_dir = home_dir.join(".cache");
            if user_temp_dir.exists() {
                total_size += calculate_directory_size(&user_temp_dir)?;
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
