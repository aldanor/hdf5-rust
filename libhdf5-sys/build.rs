extern crate pkg_config;
extern crate tempdir;

use std::env;
use std::ffi::OsString;
use std::process::Command;
use std::str;

use tempdir::TempDir;

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

fn get_hdf5_version(libs: &[String], dirs: &[String]) -> (u8, u8, u8) {
    let src = env::current_dir().unwrap().join("hdf5_version.rs");
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let dir = TempDir::new_in(".", "tmp").unwrap();

    let dst = dir.path().to_path_buf();
    let mut cmd = Command::new(&rustc);
    cmd.arg(&src);
    for lib in libs.iter() {
        cmd.arg("-l").arg(lib);
    }
    for dir in dirs.iter() {
        cmd.arg("-L").arg(dir);
    }
    cmd.arg("--out-dir").arg(&dst);

    let out: Result<_, String> = (|| {
        let out = try!(cmd.output().map_err(|e| format!("{:?}", e)));
        if !out.status.success() {
            return Err(format!("`{:?}` failed:\n{}",
                               cmd, unsafe { str::from_utf8_unchecked(&out.stderr) }));
        }
        let bin = dst.join("hdf5_version");
        Command::new(&bin).output().map_err(|e| format!("{:?}", e))
    })();
    dir.close().unwrap();
    let out = out.unwrap();
    let stdout = str::from_utf8(&out.stdout).unwrap();

    let version: Vec<_> = stdout.split_whitespace().map(|s| s.parse::<u8>().unwrap()).collect();
    assert_eq!(version.len(), 3);

    (version[0], version[1], version[2])
}

fn main() {
    let (libs, dirs) = find_hdf5_libs();
    let version = get_hdf5_version(&libs, &dirs);

    assert!(version >= (1, 8, 0));
    if version >= (1, 8, 14) {
        println!("cargo:rustc-cfg=hdf5_1_8_14");
    }

    for dir in dirs.iter() {
        println!("cargo:rustc-flags=-L {}", dir);
    }
    for lib in libs.iter() {
        println!("cargo:rustc-flags=-l {}", lib);
    }
}
