use std::path::PathBuf;

use tempfile::tempdir;

use crate::internal_prelude::*;

pub fn with_tmp_dir<F: Fn(PathBuf)>(func: F) {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    func(path);
}

pub fn with_tmp_path<F: Fn(PathBuf)>(func: F) {
    with_tmp_dir(|dir| func(dir.join("foo.h5")))
}

pub fn with_tmp_file<F: Fn(File)>(func: F) {
    with_tmp_path(|path| {
        let file = File::create(&path).unwrap();
        func(file);
    })
}
