// See LICENSE file for copyright and license details.

use cgmath::vector::Vec2;

#[deriving(Decodable)]
pub struct Size2<T> {
    x: T,
    y: T,
}

pub type Bool = bool;
pub type Int = i32; // TODO: rename, collision with trait

#[deriving(Ord, Eq, Hash)]
pub struct PlayerId(Int);

#[deriving(Ord, Eq, Hash)]
pub struct UnitId(Int);

pub type MapPos = Vec2<Int>;

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
