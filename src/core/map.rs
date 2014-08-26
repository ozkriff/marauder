// See LICENSE file for copyright and license details.

use std::num::{abs};
use cgmath::{Vector2};
use core::types::{Size2, MInt, MapPos};

pub struct MapPosIter {
    cursor: MapPos,
    map_size: Size2<MInt>,
}

impl MapPosIter {
    pub fn new(map_size: Size2<MInt>) -> MapPosIter {
        MapPosIter {
            cursor: MapPos{v: Vector2::zero()},
            map_size: map_size,
        }
    }
}

impl Iterator<MapPos> for MapPosIter {
    fn next(&mut self) -> Option<MapPos> {
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
    (abs(dx) + abs(dy) + abs(dx - dy)) / 2
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
