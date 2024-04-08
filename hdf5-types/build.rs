fn main() {
    let print_feature = |key: &str| println!("cargo:rustc-cfg=feature=\"{}\"", key);
    let print_cfg = |key: &str| println!("cargo:rustc-cfg={}", key);
    println!("cargo:rerun-if-changed=build.rs");
    for (key, _) in std::env::vars() {
        match key.as_str() {
            "DEP_HDF5_MSVC_DLL_INDIRECTION" => print_cfg("windows_dll"),
            key if key.starts_with("DEP_HDF5_VERSION_") => {
                print_feature(&key.trim_start_matches("DEP_HDF5_VERSION_").replace('_', "."));
            }
            _ => continue,
        }
    }
}
