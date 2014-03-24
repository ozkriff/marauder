// See LICENSE file for copyright and license details.

use cgmath::vector::Vec2;
use std::cmp::{TotalOrd};

#[deriving(Decodable)]
pub struct Size2<T>{w: T, h: T}

pub type MBool = bool;
pub type MInt = i32;

#[deriving(Ord, Eq, TotalEq, Hash)]
pub struct PlayerId(MInt);

#[deriving(Ord, TotalOrd, Eq, TotalEq, Hash)]
pub struct UnitId(MInt);

pub type MapPos = Vec2<MInt>;

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
