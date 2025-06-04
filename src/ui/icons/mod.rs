use eframe::egui::IconData;
use image::{load_from_memory, ImageBuffer, Rgba};

// Créer une icône d'application à partir d'un fichier image
pub fn create_app_icon(logo_bytes: &[u8]) -> IconData {
    // Utiliser le logo intégré
    if let Ok(image) = load_from_memory(logo_bytes) {
        // Redimensionner l'image si elle est trop grande
        let max_dimension = 128; // Taille maximale pour l'icône
        let width = std::cmp::min(image.width(), max_dimension);
        let height = std::cmp::min(image.height(), max_dimension);
        
        // Utiliser l'image redimensionnée
        let image = image.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        let rgba = image.to_rgba8().into_raw();
        
        return IconData {
            rgba,
            width,
            height,
        };
    }
    
    // Si le chargement échoue, créer une icône par défaut
    // Créer une image de 32x32 pixels
    let width = 32;
    let height = 32;
    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

    // Générer un dégradé bleu avec un motif rappelant la mémoire
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        // Créer un dégradé du centre vers l'extérieur
        let dx = (x as f32 / width as f32 - 0.5) * 2.0;
        let dy = (y as f32 / height as f32 - 0.5) * 2.0;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // Couleur de base bleu
        let mut r = 30;
        let mut g = 144;
        let mut b = 255;
        
        // Ajuster l'intensité en fonction de la distance
        let intensity = (1.0 - distance).max(0.0);
        r = (r as f32 * intensity) as u8;
        g = (g as f32 * intensity) as u8;
        b = (b as f32 * intensity) as u8;
        
        // Ajouter un motif de "circuit" pour représenter la mémoire
        if (x % 8 == 0 || y % 8 == 0) && distance < 0.9 {
            r = (r as f32 * 1.2).min(255.0) as u8;
            g = (g as f32 * 1.2).min(255.0) as u8;
            b = (b as f32 * 1.2).min(255.0) as u8;
        }
        
        *pixel = Rgba([r, g, b, 255]);
    }

    // Convertir l'image en RGBA pour egui
    let rgba = img.into_raw();
    
    IconData {
        rgba,
        width,
        height,
    }
} 