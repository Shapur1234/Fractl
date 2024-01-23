fn main() {
    if cfg!(feature = "gpu") && std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|target_os| target_os == "linux") {
        println!("cargo:rustc-link-lib=vulkan");
    }
}
