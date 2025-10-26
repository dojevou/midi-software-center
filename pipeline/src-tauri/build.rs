fn main() {
    // Tell the linker to use webkit2gtk-4.1 instead of webkit2gtk-4.0
    println!("cargo:rustc-link-lib=webkit2gtk-4.1");
    println!("cargo:rustc-link-lib=javascriptcoregtk-4.1");

    tauri_build::build()
}
