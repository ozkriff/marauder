// See LICENSE file for copyright and license details.

#[deriving(Decodable)]
pub struct Size2<T> {
  x: T,
  y: T,
}

pub type Bool = bool;
pub type Int = i32;

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
