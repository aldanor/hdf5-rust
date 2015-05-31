use file::File;

use std::path::{Path, PathBuf};
use tempdir::TempDir;

pub fn with_tmp_dir<F: Fn(PathBuf)>(func: F) {
    let dir = TempDir::new_in(".", "tmp").unwrap();
    let path = dir.path().to_path_buf();
    func(path);
}

pub fn with_tmp_path<P: AsRef<Path>, F: Fn(PathBuf)>(path: P, func: F) {
    with_tmp_dir(|dir| {
        func(dir.join(path.as_ref()))
    })
}

pub fn with_tmp_file<F: Fn(File)>(func: F) {
    with_tmp_path("foo.h5", |path| {
        let file = File::open(&path, "w").unwrap();
        func(file);
    })
}
