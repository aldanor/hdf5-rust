use std::env;

#[cfg(feature = "lzf")]
fn build_lzf() {
    cc::Build::new()
        .warnings(false)
        .opt_level(3)
        .file("ext/lzf/lzf_c.c")
        .file("ext/lzf/lzf_d.c")
        .include("ext/lzf")
        .compile("lzf");
}

#[cfg(feature = "blosc")]
fn build_blosc() {
    use std::env;

    let is_true = |x: &str| x == "1" || x == "yes" || x == "true";
    let check_env = |k: &str| is_true(&env::var(k).unwrap_or_default().to_ascii_lowercase());

    let mut cfg = cmake::Config::new("ext/c-blosc");
    if check_env("BLOSC_NO_SSE2") {
        cfg.define("DEACTIVATE_SSE2", "1");
    }
    if check_env("BLOSC_NO_AVX2") {
        cfg.define("DEACTIVATE_AVX2", "1");
    }
    let dst = cfg
        .uses_cxx11()
        .always_configure(true)
        .no_build_target(true)
        .very_verbose(true)
        .define("BUILD_TESTS", "0")
        .define("BUILD_BENCHMARKS", "0")
        .define("BUILD_SHARED", "0")
        .build();

    println!("cargo:rustc-link-search=native={}", dst.join("build/blosc").display());
    println!("cargo:rustc-link-lib=static=blosc");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-flags=-l dylib=c++");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-flags=-l dylib=libstdc++");
    }
}

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
    #[cfg(feature = "lzf")]
    build_lzf();
    #[cfg(feature = "blosc")]
    build_blosc();
}
