fn main() {
    // 前端 dist、Tauri 配置和图标变化时重新运行 build script，确保桌面包嵌入最新资源。
    println!("cargo:rerun-if-changed=../dist");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=icons");
    tauri_build::build();
}
