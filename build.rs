fn main() {
    // Ensure libsteam_api.so can be found
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
}
