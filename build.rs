//! Tell the linker to link to the [Vulkan](https://registry.khronos.org/vulkan/) library.
fn main() {
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=vulkan");
    }
}
