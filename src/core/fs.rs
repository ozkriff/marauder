// See LICENSE file for copyright and license details.

use std::os;

pub struct FileSystem {
    root_path: Path,
}

impl FileSystem {
    pub fn new() -> FileSystem {
        FileSystem {
            root_path: os::self_exe_path().unwrap(),
        }
    }

    pub fn get(&self, path: &Path) -> Path {
        self.root_path.join(path)
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
