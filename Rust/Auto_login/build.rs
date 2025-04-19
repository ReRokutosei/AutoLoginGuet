use std::path::Path;

fn main() {
    if cfg!(target_os = "windows") {
        let icon_path = "resources/Rust.ico";
        
        // 检查图标文件是否存在
        if !Path::new(icon_path).exists() {
            println!("cargo:warning=Icon file not found at: {}", icon_path);
            return;
        }

        let mut res = winres::WindowsResource::new();
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="asInvoker" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#);
        res.set_manifest_file("manifest.xml");
        if let Err(e) = res.compile() {
            eprintln!("Failed to set up Windows resource: {}", e);
        }

        if let Err(e) = try_build_resources(icon_path) {
            println!("cargo:warning=Failed to build resources: {}", e);
        }
    }
}

fn try_build_resources(icon_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut res = winres::WindowsResource::new();
    res.set_icon(icon_path);
    res.compile()?;
    Ok(())
}
