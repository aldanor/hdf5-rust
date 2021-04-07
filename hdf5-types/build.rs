fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("DEP_HDF5_MSVC_DLL_INDIRECTION").is_some() {
        println!("cargo:rustc-cfg=windows_dll");
    }
}
