// Thumbnails cleaning

use anyhow::Result;
use std::fs;
use std::path::{Path};
use walkdir::WalkDir;

pub async fn clean_thumbnails() -> Result<u64> {
    let mut total_cleaned = 0u64;

    #[cfg(target_os = "windows")]
    {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let thumbnails_dirs_str = vec![
                format!("{}\\AppData\\Local\\Microsoft\\Windows\\Explorer", user_profile),
                format!("{}\\AppData\\Local\\Packages\\Microsoft.Windows.Photos_8wekyb3d8bbwe\\LocalState\\PhotosAppCache", user_profile),
            ];

            for thumb_dir_str in thumbnails_dirs_str {
                let path = Path::new(&thumb_dir_str);
                if path.exists() {
                    total_cleaned += clean_thumbnails_directory(path).await?;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(home_dir) = dirs::home_dir() {
            let thumbnails_dirs_path = vec![
                home_dir.join(".cache/thumbnails"),
                home_dir.join(".thumbnails"), // Ancien emplacement, parfois encore utilisé
            ];
            for path in thumbnails_dirs_path {
                if path.exists() {
                    total_cleaned += clean_thumbnails_directory(&path).await?;
                }
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
            
            // Pour Linux, les fichiers de miniatures sont souvent des .png ou .jpeg directement.
            // La fonction is_thumbnail_file est très spécifique à Windows.
            // Nous allons simplement supprimer les fichiers dans les répertoires de miniatures pour Linux pour l'instant.
            // Une approche plus sûre serait de vérifier les types MIME ou les extensions courantes d'images.
            #[cfg(target_os = "windows")]
            let should_delete = is_thumbnail_file(path);
            #[cfg(not(target_os = "windows"))]
            let should_delete = true; // Simplification pour Linux : nettoie tout dans les dossiers de miniatures.

            if should_delete {
                if let Ok(metadata) = entry.metadata() {
                    let file_size = metadata.len();
                    if fs::remove_file(path).is_ok() {
                        total_size += file_size;
                    }
                }
            }
        }
    }
    Ok(total_size)
}

#[cfg(target_os = "windows")]
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

    #[cfg(target_os = "windows")]
    {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let thumbnails_dirs_str = vec![
                format!("{}\\AppData\\Local\\Microsoft\\Windows\\Explorer", user_profile),
                format!("{}\\AppData\\Local\\Packages\\Microsoft.Windows.Photos_8wekyb3d8bbwe\\LocalState\\PhotosAppCache", user_profile),
            ];

            for thumb_dir_str in thumbnails_dirs_str {
                let path = Path::new(&thumb_dir_str);
                if path.exists() {
                    total_size += calculate_thumbnails_size_os(path)?;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(home_dir) = dirs::home_dir() {
            let thumbnails_dirs_path = vec![
                home_dir.join(".cache/thumbnails"),
                home_dir.join(".thumbnails"),
            ];
            for path in thumbnails_dirs_path {
                if path.exists() {
                    total_size += calculate_thumbnails_size_os(&path)?;
                }
            }
        }
    }
    Ok(total_size)
}

// Renommée pour éviter la collision avec la version spécifique à Windows
fn calculate_thumbnails_size_os(dir: &Path) -> Result<u64> {
    let mut total_size = 0u64;
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        #[cfg(target_os = "windows")]
        let is_thumb = entry.file_type().is_file() && is_thumbnail_file(entry.path());
        #[cfg(not(target_os = "windows"))]
        let is_thumb = entry.file_type().is_file(); // Simplification pour Linux

        if is_thumb {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }
    Ok(total_size)
}
