fn main() {
    if cfg!(all(feature = "gpu", target_os = "linux")) {
        println!("cargo:rustc-link-lib=vulkan");
    }
}
