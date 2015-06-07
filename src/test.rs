use file::File;

use std::path::PathBuf;
use tempdir::TempDir;

pub fn with_tmp_dir<F: Fn(PathBuf)>(func: F) {
    let dir = TempDir::new_in(".", "tmp").unwrap();
    let path = dir.path().to_path_buf();
    func(path);
}

pub fn with_tmp_path<F: Fn(PathBuf)>(func: F) {
    with_tmp_dir(|dir| {
        func(dir.join("foo.h5"))
    })
}

pub fn with_tmp_file<F: Fn(File)>(func: F) {
    with_tmp_path(|path| {
        let file = File::open(&path, "w").unwrap();
        func(file);
    })
}
