fn main() {
    // Re-run the build script when bundled desktop assets or the Windows manifest change.
    println!("cargo:rerun-if-changed=../dist");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=icons");
    println!("cargo:rerun-if-changed=app.manifest");

    let windows = tauri_build::WindowsAttributes::new().app_manifest(include_str!("app.manifest"));
    let attrs = tauri_build::Attributes::new().windows_attributes(windows);

    tauri_build::try_build(attrs).expect("failed to run Tauri build script");
}
