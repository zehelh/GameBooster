use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Obtenez le répertoire de sortie de Cargo (ex: target/release)
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("../../..");

    // Chemin vers le NSudo.exe dans le dossier resources
    let nsudo_src_path = Path::new("resources/NSudo.exe");

    // Copiez NSudo.exe s'il existe
    if nsudo_src_path.exists() {
        let nsudo_dest_path = dest_path.join("NSudo.exe");
        println!("cargo:rerun-if-changed=resources/NSudo.exe");
        fs::copy(nsudo_src_path, nsudo_dest_path).expect("Failed to copy NSudo.exe");
    } else {
        // Affichez un message d'avertissement si NSudo.exe n'est pas trouvé
        println!(
            "cargo:warning=NSudo.exe not found in 'resources' directory. \
            Please download it and place it there for the TrustedInstaller feature to work."
        );
    }

    // Copiez les fichiers WinDivert
    let windivert_files = ["WinDivert.dll", "WinDivert64.sys"];
    let windivert_src_dir = Path::new("resources/windivert");

    if windivert_src_dir.exists() {
        for file_name in &windivert_files {
            let src_path = windivert_src_dir.join(file_name);
            if src_path.exists() {
                let dest_file_path = dest_path.join(file_name);
                println!("cargo:rerun-if-changed={}", src_path.display());
                fs::copy(&src_path, dest_file_path).expect("Failed to copy WinDivert file");
            } else {
                 println!(
                    "cargo:warning={} not found in 'resources/windivert' directory.", file_name
                );
            }
        }
    } else {
         println!(
            "cargo:warning='resources/windivert' directory not found. \
            The network features will not work correctly."
        );
    }
} 