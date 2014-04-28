// See LICENSE file for copyright and license details.

use cgmath::vector::Vector2;
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

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
