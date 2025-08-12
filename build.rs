use std::env;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        generate_windows_resource();
    }

    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
}

#[cfg(windows)]
fn generate_windows_resource() {
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "1.0.0".to_string());
    
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico")
        .set_language(0x0409)
        .set_manifest_file("assets/manifest.xml");
    
    res.set("FileDescription", "AutoLoginGUET")
        .set("ProductName", "AutoLoginGUET")
        .set("ProductVersion", &version)
        .set("FileVersion", &version)
        .set("CompanyName", "© 2025 ReRokutosei")
        .set("LegalCopyright", "By ReRokutosei. All rights reserved.")
        .set("OriginalFilename", "AutoLoginGUET.exe");
    
    res.compile().unwrap();
}