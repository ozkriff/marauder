// See LICENSE file for copyright and license details.

use crate::core::dir::Dir::{East, NorthEast, NorthWest, SouthEast, SouthWest, West};
use crate::core::types::{MInt, MapPos};
use cgmath::Vector2;

#[derive(Copy, Clone)]
pub enum Dir {
    NorthEast,
    East,
    SouthEast,
    SouthWest,
    West,
    NorthWest,
}

const DIR_TO_POS_DIFF: [[Vector2<MInt>; 6]; 2] = [
    [
        Vector2 { x: 1, y: -1 },
        Vector2 { x: 1, y: 0 },
        Vector2 { x: 1, y: 1 },
        Vector2 { x: 0, y: 1 },
        Vector2 { x: -1, y: 0 },
        Vector2 { x: 0, y: -1 },
    ],
    [
        Vector2 { x: 0, y: -1 },
        Vector2 { x: 1, y: 0 },
        Vector2 { x: 0, y: 1 },
        Vector2 { x: -1, y: 1 },
        Vector2 { x: -1, y: 0 },
        Vector2 { x: -1, y: -1 },
    ],
];

impl Dir {
    pub fn from_int(n: MInt) -> Dir {
        assert!(n >= 0 && n < 6);
        let dirs = [NorthEast, East, SouthEast, SouthWest, West, NorthWest];
        dirs[n as usize]
    }

    pub fn to_int(&self) -> MInt {
        match *self {
            NorthEast => 0,
            East => 1,
            SouthEast => 2,
            SouthWest => 3,
            West => 4,
            NorthWest => 5,
        }
    }

    pub fn get_dir_from_to(from: MapPos, to: MapPos) -> Dir {
        // assert!(from.distance(to) == 1);
        let diff = to.v - from.v;
        for i in 0..6 {
            if diff == DIR_TO_POS_DIFF[(from.v.y % 2) as usize][i] {
                return Dir::from_int(i as MInt);
            }
        }
        panic!("impossible positions: {:?}, {:?}", from, to);
    }

    pub fn get_neighbour_pos(pos: MapPos, dir: Dir) -> MapPos {
        let is_odd_row = pos.v.y % 2 == 1;
        let subtable_index = if is_odd_row { 1 } else { 0 };
        let direction_index = dir.to_int();
        assert!(direction_index >= 0 && direction_index < 6);
        let difference = DIR_TO_POS_DIFF[subtable_index][direction_index as usize];
        MapPos {
            v: pos.v + difference,
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
