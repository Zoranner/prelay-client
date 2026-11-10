#[cfg(windows)]
use std::{fs, path::Path};

mod discovery;

#[cfg(windows)]
pub(super) fn write(path: impl AsRef<Path>, contents: &str) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}
