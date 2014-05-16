// See LICENSE file for copyright and license details.

use std::f32::consts::PI;
use std::str::from_utf8_owned;
use std::io::File;
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
    let bytes = match File::open(path).read_to_end() {
        Ok(bytes) => bytes.as_slice().to_owned(),
        Err(msg) => fail!("Can not read from file {}: {}", path.display(), msg),
    };
    match from_utf8_owned(bytes) {
        Some(s) => s,
        None => fail!("Not valid utf8 in file {}", path.display()),
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
