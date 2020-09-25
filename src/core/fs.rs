// See LICENSE file for copyright and license details.

use std::path::{Path, PathBuf};

pub struct FileSystem {
    root_path: PathBuf,
}

impl FileSystem {
    pub fn new() -> FileSystem {
        FileSystem {
            root_path: std::env::current_dir().unwrap(),
        }
    }

    pub fn get(&self, path: &Path) -> PathBuf {
        self.root_path.join(path)
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
