extern crate pkg_config;

use std::env;

fn find_hdf5_libs() -> (Vec<String>, Vec<String>) {
    let (mut libs, mut dirs) = (vec![], vec![]);

    if let Ok(libname) = env::var("HDF5_LIBNAME") {
        libs.push(libname);
    }
    if let Ok(libdir) = env::var("HDF5_LIBDIR") {
        dirs.push(libdir);
    }

    if let Ok(library) = pkg_config::Config::new().find("hdf5") {
        if dirs.is_empty() {
            for dir in library.link_paths.iter() {
                dirs.push(dir.to_str().unwrap().to_owned());
            }
        }
        if libs.is_empty() {
            for lib in library.libs.iter() {
                libs.push(lib.clone());
            }
        }
    }

    if libs.is_empty() {
        libs.push("hdf5".to_owned());
    }

    (libs, dirs)
}

fn main() {
    let (libs, dirs) = find_hdf5_libs();

    for dir in dirs.iter() {
        println!("cargo:rustc-flags=-L {}", dir);
    }
    for lib in libs.iter() {
        println!("cargo:rustc-flags=-l {}", lib);
    }
}
