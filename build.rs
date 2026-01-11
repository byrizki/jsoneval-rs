fn main() {
    // Add version resource to Windows DLL
    // IMPORTANT: Check CARGO_CFG_TARGET_OS to detect what we're building FOR (target),
    // not what we're building ON (host). This is crucial for cross-compilation.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    
    if target_os == "windows" {
        let version = env!("CARGO_PKG_VERSION");
        
        // Parse version string (e.g., "0.0.49" -> parts [0, 0, 49])
        let version_parts: Vec<u64> = version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        let major = version_parts.get(0).copied().unwrap_or(0);
        let minor = version_parts.get(1).copied().unwrap_or(0);
        let patch = version_parts.get(2).copied().unwrap_or(0);
        
        let mut res = winres::WindowsResource::new();
        
        // Set the numeric version (this is what Windows Explorer shows)
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, (major << 48) | (minor << 32) | (patch << 16));
        res.set_version_info(winres::VersionInfo::FILEVERSION, (major << 48) | (minor << 32) | (patch << 16));
        
        // Set string version information
        res.set("ProductName", "JSON Eval RS")
            .set("FileDescription", "High-performance JSON Logic evaluator with schema validation")
            .set("CompanyName", "Quadrant Synergy International")
            .set("LegalCopyright", "Copyright © 2024 Muhamad Rizki")
            .set("ProductVersion", version)
            .set("FileVersion", version)
            .set("OriginalFilename", "json_eval_rs.dll")
            .set("InternalName", "json_eval_rs");
        
        if let Err(e) = res.compile() {
            eprintln!("Error: Failed to compile Windows resource: {}", e);
            // Don't panic, just warn. Dependencies might be missing on some setups.
            println!("cargo:warning=Failed to compile Windows resource: {}", e);
        }
    }
    
    // Print version info for all platforms
    println!("cargo:rustc-env=BUILD_VERSION={}", env!("CARGO_PKG_VERSION"));
    println!("cargo:rerun-if-changed=Cargo.toml");
}
