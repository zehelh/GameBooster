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

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=app.rc"); // Added for the new .rc file
    // embed_resource::compile("app.manifest", embed_resource::NONE); // Old line
    embed_resource::compile("app.rc", embed_resource::NONE); // Compile app.rc instead
}