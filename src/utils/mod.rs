// Formater la taille en unités lisibles
pub fn format_size(size: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let size = size as f64;
    
    if size < KB {
        format!("{:.0} o", size)
    } else if size < MB {
        format!("{:.1} Ko", size / KB)
    } else if size < GB {
        format!("{:.1} Mo", size / MB)
    } else {
        format!("{:.2} Go", size / GB)
    }
} 