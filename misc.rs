// See LICENSE file for copyright and license details.

use std;
use cgmath::matrix::{
  Mat4,
  Matrix
};

pub fn print_mat4(name: &str, mat: Mat4<f32>) {
  println!("{}:", name);
  for i in range(0u, 4) {
    for j in range(0u, 4) {
      print!("{} ", *mat.cr(i, j));
    }
    println!("");
  }
  println!("");
}

/// usage: let s = mvp_matrix; println(type_of(&s));
pub fn type_of<T>(_: &T) -> &'static str {
  unsafe {
    (*std::unstable::intrinsics::get_tydesc::<T>()).name
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
