fn main() {
    #[cfg(target_os = "windows")]
    {
        // Set Windows subsystem to hide console
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
        
        // Embed icon using embed-resource
        embed_resource::compile("icon.rc", embed_resource::NONE);
    }
}
