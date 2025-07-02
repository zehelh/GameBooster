use std::env;

// This module will only be compiled when the target OS is Windows.
// The `#[cfg(target_os = "windows")]` attribute ensures this module's contents
// are only compiled when targeting Windows.
#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::fs;
    use std::path::Path;

    pub fn setup() {
        // These println! calls tell Cargo to re-run the build script if these files change.
        println!("cargo:rerun-if-changed=app.rc");
        println!("cargo:rerun-if-changed=assets/img/logo.ico");
        println!("cargo:rerun-if-changed=src/bin/NSudo.exe");
        println!("cargo:rerun-if-changed=src/bin/WinDivert.dll");
        println!("cargo:rerun-if-changed=src/bin/WinDivert.sys");
        println!("cargo:rerun-if-changed=src/bin/WinDivert64.sys");

        // Embed the application manifest and icon.
        let _ = embed_resource::compile("app.rc", embed_resource::NONE);

        // Determine the output directory (e.g., target/release or target/debug)
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
        let profile_dir = Path::new(&out_dir)
            .ancestors()
            .nth(3) // Navigates up from target/<profile>/build/<crate-hash>/out to target/<profile>
            .unwrap_or_else(|| panic!("Could not determine profile directory from OUT_DIR: {}", out_dir));

        // Ensure the profile directory (e.g., target/release) exists.
        // This should generally be true, but it's good practice if copying to subdirectories.
        // For copying directly into target/release, it will exist.

        let files_to_copy = [
            ("src/bin/NSudo.exe", "NSudo.exe"),
            ("src/bin/WinDivert.dll", "WinDivert.dll"),
            ("src/bin/WinDivert.sys", "WinDivert.sys"),
            ("src/bin/WinDivert64.sys", "WinDivert64.sys"),
        ];

        for (src_file_rel_path, dest_file_name) in &files_to_copy {
            let src_path = Path::new(src_file_rel_path);
            if src_path.exists() {
                let dest_path = profile_dir.join(dest_file_name);
                match fs::copy(src_path, &dest_path) {
                    Ok(_) => { /* Successfully copied */ }
                    Err(e) => panic!(
                        "Failed to copy {} to {}: {}",
                        src_path.display(),
                        dest_path.display(),
                        e
                    ),
                }
            } else {
                // Optionally, print a warning if a source file is missing,
                // though rerun-if-changed might be problematic if it doesn't exist initially.
                // eprintln!("cargo:warning=Source file {} not found, skipping copy.", src_path.display());
            }
        }
    }
}

fn main() {
    // Tell Cargo to re-run this build script if build.rs itself changes.
    println!("cargo:rerun-if-changed=build.rs");

    // Get the target OS from the environment variable set by Cargo.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "windows" {
        // Only attempt to call windows::setup() if we are actually targeting Windows.
        // The `windows` module and this call are conditionally compiled via #[cfg(target_os = "windows")].
        #[cfg(target_os = "windows")]
        windows::setup();
    }
}