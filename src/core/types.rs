// See LICENSE file for copyright and license details.

use cgmath::Vector2;
use serde::Deserialize;

#[derive(Copy, Clone, Deserialize)]
pub struct Size2<T> {
    pub w: T,
    pub h: T,
}

pub type MInt = i32;

#[derive(PartialOrd, PartialEq, Eq, Hash, Clone)]
pub struct PlayerId {
    pub id: MInt,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub struct UnitId {
    pub id: MInt,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct MapPos {
    pub v: Vector2<MInt>,
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
