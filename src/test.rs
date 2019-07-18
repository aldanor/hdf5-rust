use std::path::PathBuf;

use tempdir::TempDir;

use crate::internal_prelude::*;

pub fn with_tmp_dir<F: Fn(PathBuf)>(func: F) {
    let dir = TempDir::new_in(".", "tmp").unwrap();
    let path = dir.path().to_path_buf();
    let _e = silence_errors();
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
