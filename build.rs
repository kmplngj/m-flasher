// Windows: ask for administrator rights (needed for SCSI pass-through to the
// download-mode disk) and declare the app DPI-aware.
fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        use embed_manifest::manifest::{DpiAwareness, ExecutionLevel};
        embed_manifest::embed_manifest(
            embed_manifest::new_manifest("M-Flasher")
                .requested_execution_level(ExecutionLevel::RequireAdministrator)
                .dpi_awareness(DpiAwareness::PerMonitorV2),
        )
        .expect("embedding the manifest");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
