// See LICENSE file for copyright and license details.

use std::f32::consts::PI;
use std::io::File;
use std::str::from_utf8_owned;
use gl_types::MFloat;

pub fn deg_to_rad(n: MFloat) -> MFloat {
    n * PI / 180.0
}

// TODO: handle errors
pub fn read_file(path: &Path) -> ~str {
    if !path.exists() {
        fail!("no such path");
    }
    let shader = File::open(path).map(|mut v| v.read_to_end()).unwrap();
    let shader = from_utf8_owned(shader.unwrap());
    shader.unwrap()
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
