// See LICENSE file for copyright and license details.

use cgmath::vector::Vector2;
use std::cmp::{TotalOrd};

#[deriving(Decodable)]
pub struct Size2<T>{
    pub w: T,
    pub h: T,
}

pub type Point2<T> = Vector2<T>;

pub type MInt = i32;

#[deriving(Ord, Eq, TotalEq, Hash)]
pub struct PlayerId{pub id: MInt}

#[deriving(Ord, TotalOrd, Eq, TotalEq, Hash)]
pub struct UnitId{pub id: MInt}

pub type MapPos = Vector2<MInt>;

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
