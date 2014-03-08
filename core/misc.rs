// See LICENSE file for copyright and license details.

use std::f32::consts::PI;
use std::io::File;
use std::str::from_utf8_owned;
use visualizer::types::MFloat;

pub fn deg_to_rad(n: MFloat) -> MFloat {
    n * PI / 180.0
}

pub fn rad_to_deg(n: MFloat) -> MFloat {
    (n * 180.0) / PI
}

pub fn read_file(path: &Path) -> ~str {
    if !path.exists() {
        fail!("Path does not exists: {}", path.display());
    }
    let shader = match File::open(path).map(|mut v| v.read_to_end()) {
        Ok(txt) => from_utf8_owned(match txt {
            Ok(txt) => txt,
            Err(_) => fail!("Can not read file {}", path.display()),
        }),
        Err(_) => fail!("Can not read file {}", path.display()),
    };
    match shader {
        Some(shader) => shader,
        None => fail!("Can not read file {}", path.display()),
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
