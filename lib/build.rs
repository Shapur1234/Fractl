fn main() {
    if cfg!(feature = "gpu") && cfg!(linux) {
        println!("cargo:rustc-link-lib=vulkan");
    }
}
