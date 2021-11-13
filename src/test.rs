use std::path::PathBuf;

use tempfile::tempdir;

use crate::internal_prelude::*;

pub fn with_tmp_dir<T, F: Fn(PathBuf) -> T>(func: F) -> T {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    func(path)
}

pub fn with_tmp_path<T, F: Fn(PathBuf) -> T>(func: F) -> T {
    with_tmp_dir(|dir| func(dir.join("foo.h5")))
}

pub fn with_tmp_file<T, F: Fn(File) -> T>(func: F) -> T {
    with_tmp_path(|path| {
        let file = File::create(&path).unwrap();
        func(file)
    })
}
