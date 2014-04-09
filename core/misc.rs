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

// TODO: Simplify this
pub fn read_file(path: &Path) -> ~str {
    if !path.exists() {
        fail!("Path does not exists: {}", path.display());
    }
    let shader = match File::open(path).map(|mut v| v.read_to_end()) {
        Ok(txt) => from_utf8_owned(match txt {
            Ok(txt) => txt.as_slice().to_owned(),
            Err(_) => fail!("Can not read file {}", path.display()),
        }),
        Err(_) => fail!("Can not read file {}", path.display()),
    };
    match shader {
        Some(shader) => shader,
        None => fail!("Can not read file {}", path.display()),
    }
}

pub fn add_quad_to_vec<T: Clone>(v: &mut Vec<T>, v1: T, v2: T, v3: T, v4: T) {
    v.push(v1.clone());
    v.push(v2);
    v.push(v3.clone());
    v.push(v1);
    v.push(v3);
    v.push(v4);
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
