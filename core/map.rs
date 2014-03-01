// See LICENSE file for copyright and license details.

use cgmath::vector::Vec2;
use core::types::{
    Size2,
    MInt,
    MapPos,
};

pub struct MapPosIter {
    cursor: MapPos,
    map_size: Size2<MInt>,
}

impl MapPosIter {
    pub fn new(map_size: Size2<MInt>) -> MapPosIter {
        MapPosIter {
            cursor: Vec2::zero(),
            map_size: map_size,
        }
    }
}

impl Iterator<MapPos> for MapPosIter {
    fn next(&mut self) -> Option<MapPos> {
        let current_pos = if self.cursor.y >= self.map_size.h {
            None
        } else {
            Some(self.cursor)
        };
        self.cursor.x += 1;
        if self.cursor.x >= self.map_size.w {
            self.cursor.x = 0;
            self.cursor.y += 1;
        }
        current_pos
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
