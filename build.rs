use std::env;

fn main() {
    for (key, _) in env::vars() {
        let key = match key.as_str() {
            "DEP_HDF5_HAVE_DIRECT" => "h5_have_direct".into(),
            "DEP_HDF5_HAVE_STDBOOL" => "h5_have_stdbool".into(),
            "DEP_HDF5_HAVE_PARALLEL" => "h5_have_parallel".into(),
            "DEP_HDF5_HAVE_THREADSAFE" => "h5_have_threadsafe".into(),
            "DEP_HDF5_MSVC_DLL_INDIRECTION" => "h5_dll_indirection".into(),
            key if key.starts_with("DEP_HDF5_VERSION_") => {
                let version = key.trim_start_matches("DEP_HDF5_VERSION_");
                format!("hdf5_{}", version)
            }
            _ => continue,
        };
        println!("cargo:rustc-cfg={}", key);
    }
}
