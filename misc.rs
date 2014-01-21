// See LICENSE file for copyright and license details.

use std;

/// usage: let f = 1.0f32; println(type_of(&f));
pub fn type_of<T>(_: &T) -> &'static str {
  unsafe {
    (*std::unstable::intrinsics::get_tydesc::<T>()).name
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
