// See LICENSE file for copyright and license details.

use cgmath::vector::Vector2;
use std::cmp::Ord;

#[deriving(Decodable)]
pub struct Size2<T>{
    pub w: T,
    pub h: T,
}

pub struct Point2<T>{pub v: Vector2<T>}

pub type MInt = i32;

#[deriving(PartialOrd, PartialEq, Eq, Hash)]
pub struct PlayerId{pub id: MInt}

#[deriving(PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct UnitId{pub id: MInt}

pub struct SlotId{pub id: MInt}

#[deriving(PartialEq, Clone, Show)]
pub struct MapPos{pub v: Vector2<MInt>}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
