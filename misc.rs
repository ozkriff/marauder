// See LICENSE file for copyright and license details.

use std::f32::consts::PI;
use gl::types::GLchar;

pub fn c_str(s: &str) -> *GLchar {
  unsafe {
    s.to_c_str().unwrap()
  }
}

pub fn deg_to_rad(n: f32) -> f32 {
  n * PI / 180.0
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
