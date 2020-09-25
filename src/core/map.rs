// See LICENSE file for copyright and license details.

use crate::core::types::{MInt, MapPos, Size2};
use cgmath::{Vector, Vector2};

#[derive(Copy, Clone)]
pub struct MapPosIter {
    cursor: MapPos,
    map_size: Size2<MInt>,
}

impl MapPosIter {
    pub fn new(map_size: Size2<MInt>) -> MapPosIter {
        MapPosIter {
            cursor: MapPos { v: Vector2::zero() },
            map_size,
        }
    }
}

impl Iterator for MapPosIter {
    type Item = MapPos;

    fn next(&mut self) -> Option<Self::Item> {
        let current_pos = if self.cursor.v.y >= self.map_size.h {
            None
        } else {
            Some(self.cursor)
        };
        self.cursor.v.x += 1;
        if self.cursor.v.x >= self.map_size.w {
            self.cursor.v.x = 0;
            self.cursor.v.y += 1;
        }
        current_pos
    }
}

pub fn distance(from: MapPos, to: MapPos) -> MInt {
    let to = to.v;
    let from = from.v;
    let dx = (to.x + to.y / 2) - (from.x + from.y / 2);
    let dy = to.y - from.y;
    (dx.abs() + dy.abs() + (dx - dy).abs()) / 2
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
