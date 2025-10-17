use std::env;

fn main() {
    // Add version resource to Windows DLL
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico")
            .set("ProductName", "JSON Eval RS")
            .set("FileDescription", "High-performance JSON Logic evaluator with schema validation")
            .set("CompanyName", "Quadrant Synergy International")
            .set("LegalCopyright", "Copyright © 2024 Muhamad Rizki")
            .set("ProductVersion", env!("CARGO_PKG_VERSION"))
            .set("FileVersion", env!("CARGO_PKG_VERSION"))
            .set("OriginalFilename", "json_eval_rs.dll")
            .set("InternalName", "json_eval_rs");
        
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resource: {}", e);
        }
    }
    
    // Print version info for other platforms
    println!("cargo:rustc-env=BUILD_VERSION={}", env!("CARGO_PKG_VERSION"));
    println!("cargo:rerun-if-changed=Cargo.toml");
}
