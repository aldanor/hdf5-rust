use crypto::md5;
use crypto::digest::Digest;

use curl::easy::Easy;
use bzip2::read::BzDecoder;
use tar::Archive;

use std::env::{self, var};
use std::path::*;
use std::fs::{self, File};
use std::io::*;

#[cfg(target_env = "msvc")]
const IS_MSVC: bool = true;
#[cfg(not(target_env = "msvc"))]
const IS_MSVC: bool = false;

#[cfg(target_os = "windows")]
const IS_WINDOWS: bool = true;
#[cfg(not(target_os = "windows"))]
const IS_WINDOWS: bool = false;

macro_rules! ok_or_continue {
    ($r:expr) => {
        match $r {
            Err(_) => continue,
            Ok(ok) => ok,
        }
    };
}

macro_rules! some_or_continue {
    ($r:expr) => {
        match $r {
            None => continue,
            Some(some) => some,
        }
    };
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

    if let Ok(library) = pkg_config::Config::new().probe("hdf5") {
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
    conda_static();
    return;
    if var("CARGO_FEATURE_CONDA").is_ok() {
        conda_static();
    } else {
        let (libs, dirs) = find_hdf5_libs();

        for dir in dirs.iter() {
            println!("cargo:rustc-link-search={}", dir);
        }
        for lib in libs.iter() {
            println!("cargo:rustc-link-lib={}", lib);
        }
    }
}


// Use `conda search --json --platform 'win-64' mkl-static`
// to query the metadata of conda package (includes MD5 sum).

#[cfg(target_os = "linux")]
mod conda {
    pub const LIB_PATH: &'static str = "lib";

    pub const DLS: &[(&'static str, &'static str, &'static str)] = &[
        ("hdf5-1.10.4-hb1b8bf9_0.tar.bz2", 
         "https://repo.continuum.io/pkgs/main/linux-64/hdf5-1.10.4-hb1b8bf9_0.tar.bz2",
         "e25e1d2af9836593f3678198b14816eb")
    ];
}

#[cfg(target_os = "macos")]
mod conda {
    pub const LIB_PATH: &'static str = "lib";

    pub const DLS: &[(&'static str, &'static str, &'static str)] = &[
        ("hdf5-1.8.20-hfa1e0ec_1.tar.bz2", 
         "https://repo.continuum.io/pkgs/main/osx-64/hdf5-1.8.20-hfa1e0ec_1.tar.bz2", 
         "6b7457d9be3293d8ba73c36a0915d5f6")
    ];
}

#[cfg(target_os = "windows")]
mod conda {
    pub const LIB_PATH: &'static str = "Library\\lib";

    pub const DLS: &[(&'static str, &'static str, &'static str)] = &[
        ("hdf5-1.8.16-vc14_0.tar.bz2", 
         "https://repo.continuum.io/pkgs/free/win-64/hdf5-1.8.16-vc14_0.tar.bz2", 
         "c935a1d232cbe8fe09c1ffe0a64a322b")
    ];
}

fn download(uri: &str, filename: &str, out_dir: &Path) {

    let out = PathBuf::from(out_dir.join(filename));

    // Download the tarball.
    let f = File::create(&out).unwrap();
    let mut writer = BufWriter::new(f);
    let mut easy = Easy::new();
    easy.follow_location(true).unwrap();
    easy.url(&uri).unwrap();
    easy.write_function(move |data| {
        Ok(writer.write(data).unwrap())
    }).unwrap();
    easy.perform().unwrap();

    let response_code = easy.response_code().unwrap();
    if response_code != 200 {
        panic!("Unexpected response code {} for {}", response_code, uri);
    }
}

fn calc_md5(path: &Path) -> String {
    let mut sum = md5::Md5::new();
    let mut f = BufReader::new(fs::File::open(path).unwrap());
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    sum.input(&buf);
    sum.result_str()
}

fn extract<P: AsRef<Path>, P2: AsRef<Path>>(archive_path: P, extract_to: P2) {
    let file = File::open(archive_path).unwrap();
    let unzipped = BzDecoder::new(file);
    let mut a = Archive::new(unzipped);
    a.unpack(extract_to).unwrap();
}

fn conda_static() {
    let out_dir = PathBuf::from(var("OUT_DIR").unwrap());

    for (archive, uri, md5) in conda::DLS {
        let archive_path = out_dir.join(archive);
        if archive_path.exists() && calc_md5(&archive_path) == *md5 {
            println!("Use existings archive");
        } else {
            println!("Download archive");
            download(uri, archive, &out_dir);
            extract(&archive_path, &out_dir);
            
            let sum = calc_md5(&archive_path);
            if sum != *md5 {
                panic!(
                    "check sum of downloaded archive is incorrect: md5sum={}",
                    sum
                );
            }
        }
    }
    
    println!("cargo:rustc-link-search={}", out_dir.join(conda::LIB_PATH).display());
    println!("cargo:rustc-link-lib=static=hdf5");
    println!("cargo:rustc-link-lib=static=z")
}