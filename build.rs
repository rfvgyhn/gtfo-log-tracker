use winresource::WindowsResource;

fn main() {
    // Ensure libsteam_api.so can be found
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "windows" {
        compile_win_icon();
    }
}

fn compile_win_icon() {
    let mut res = WindowsResource::new();
    let icon_path = "resources/icon.ico";

    res.set_icon(icon_path);
    res.compile().expect("failed to build executable icon");
}
