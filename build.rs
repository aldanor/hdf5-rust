use std::env;

fn main() {
    for (key, _) in env::vars() {
        let key = match key.as_str() {
            "DEP_HDF5_HAVE_DIRECT" => "have_direct".into(),
            "DEP_HDF5_HAVE_STDBOOL" => "have_stdbool".into(),
            "DEP_HDF5_HAVE_PARALLEL" => "have_parallel".into(),
            "DEP_HDF5_HAVE_THREADSAFE" => "have_threadsafe".into(),
            "DEP_HDF5_MSVC_DLL_INDIRECTION" => "dll_indirection".into(),
            key if key.starts_with("DEP_HDF5_VERSION_") => {
                let version = key.trim_start_matches("DEP_HDF5_VERSION_");
                format!("feature=\"{}\"", version.replace("_", "."))
            }
            _ => continue,
        };
        println!("cargo:rustc-cfg={}", key);
    }
}
