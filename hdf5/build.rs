use std::env;

fn main() {
    let print_feature = |key: &str| println!("cargo:rustc-cfg=feature=\"{}\"", key);
    let print_cfg = |key: &str| println!("cargo:rustc-cfg={}", key);
    for (key, _) in env::vars() {
        match key.as_str() {
            // public features
            "DEP_HDF5_HAVE_DIRECT" => print_feature("have-direct"),
            "DEP_HDF5_HAVE_PARALLEL" => print_feature("have-parallel"),
            "DEP_HDF5_HAVE_THREADSAFE" => print_feature("have-threadsafe"),
            // internal config flags
            "DEP_HDF5_MSVC_DLL_INDIRECTION" => print_cfg("msvc_dll_indirection"),
            // public version features
            key if key.starts_with("DEP_HDF5_VERSION_") => {
                print_feature(&key.trim_start_matches("DEP_HDF5_VERSION_").replace('_', "."));
            }
            _ => continue,
        }
    }
}
