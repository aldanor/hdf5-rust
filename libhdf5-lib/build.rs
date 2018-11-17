use std::env;
use std::fs;
use std::path::PathBuf;

#[cfg(target_env = "msvc")]
const IS_MSVC: bool = true;
#[cfg(not(target_env = "msvc"))]
const IS_MSVC: bool = false;

#[cfg(target_os = "windows")]
const IS_WINDOWS: bool = true;
#[cfg(not(target_os = "windows"))]
const IS_WINDOWS: bool = false;

macro_rules! ok_or_continue {
    ($r:expr) => ( match $r { Err(_) => continue, Ok(ok) => ok } )
}

macro_rules! some_or_continue {
    ($r:expr) => ( match $r { None => continue, Some(some) => some } )
}

fn libdir_from_path() -> Option<String> {
    if !IS_WINDOWS || env::var("HDF5_LIBDIR").is_ok() {
        return None;
    }
    if let Ok(path) = env::var("PATH") {
        for path in path.split(";") {
            let dir = PathBuf::from(path);
            let dirname = some_or_continue!(dir.file_name());
            if dirname.to_str() != Some("bin") {
                continue;
            }
            let entries = ok_or_continue!(fs::read_dir(&dir));
            for entry in entries {
                let entry = ok_or_continue!(entry);
                let filename = entry.file_name();
                if filename.to_str() != Some("hdf5.dll") {
                    continue;
                }
                let meta = ok_or_continue!(entry.metadata());
                if !meta.is_file() {
                    continue;
                }
                if !IS_MSVC {
                    return Some(path.into());
                }
                let parent = some_or_continue!(dir.parent());
                let libdir = parent.join("lib");
                if let Some(libdir) = libdir.to_str() {
                    return Some(libdir.into());
                }
            }
        }
    }
    None
}


fn find_hdf5_libs() -> (Vec<String>, Vec<String>) {
    let (mut libs, mut dirs) = (vec![], vec![]);

    if let Ok(libname) = env::var("HDF5_LIBNAME") {
        libs.push(libname);
    }
    if let Ok(libdir) = env::var("HDF5_LIBDIR") {
        dirs.push(libdir);
    }
    if let Some(libdir) = libdir_from_path() {
        dirs.push(libdir);
    }

    if let Ok(library) = pkg_config::Config::new().find("hdf5") {
        if dirs.is_empty() {
            for dir in library.link_paths.iter() {
                dirs.push(dir.to_str().unwrap().into());
            }
        }
        if libs.is_empty() {
            for lib in library.libs.iter() {
                libs.push(lib.clone());
            }
        }
    }

    if libs.is_empty() {
        libs.push("hdf5".into());
    }

    (libs, dirs)
}

fn main() {
    let (libs, dirs) = find_hdf5_libs();

    for dir in dirs.iter() {
        println!("cargo:rustc-link-search={}", dir);
    }
    for lib in libs.iter() {
        println!("cargo:rustc-link-lib={}", lib);
    }
}
