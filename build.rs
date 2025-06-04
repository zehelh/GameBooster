// Fichier build.rs pour configurer l'application comme GUI sous Windows
// Empêche l'affichage de la console avec l'interface graphique

use std::env;
use std::fs;
use std::path::Path;

// Logo intégré en tant que ressource
const LOGO_BYTES: &[u8] = include_bytes!("./img/logo.png");

fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        
        // Créer un fichier d'icône temporaire à partir du logo intégré
        let out_dir = env::var("OUT_DIR").unwrap();
        
        // Approche simplifiée : utiliser un fichier PNG comme icône
        // Windows Resource Compiler va le convertir automatiquement
        let temp_png_path = Path::new(&out_dir).join("temp_logo.png");
        fs::write(&temp_png_path, LOGO_BYTES).unwrap();
        
        // Utiliser le fichier PNG comme icône
        res.set_icon(&temp_png_path.to_string_lossy());
        
        // Définir le sous-système comme GUI pour éviter la console
        res.set_manifest(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#,
        );
            
        if let Err(e) = res.compile() {
            eprintln!("Erreur lors de la compilation des ressources: {}", e);
        }
        
        // Force l'application à être compilée en tant qu'application Windows (GUI) 
        // pour éviter la fenêtre console
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    }

    // Indique à Cargo de reconstruire le projet si les logos changent
    println!("cargo:rerun-if-changed=img/logo.png");
} 