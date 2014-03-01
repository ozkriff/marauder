// See LICENSE file for copyright and license details.

use std::f32::consts::{
    PI,
    FRAC_PI_2,
};
use std::num::{
    sqrt,
    pow,
    sin,
    cos,
};
use cgmath::vector::{
    Vec2,
    Vec3,
    Vector,
};
use core::core_types::{
    MInt,
    MapPos,
};
use visualizer::gl_types::{
    WorldPos,
    MFloat,
    VertexCoord,
};

pub struct Geom {
    hex_ex_radius: MFloat,
    hex_in_radius: MFloat,
}

impl Geom {
    pub fn new() -> Geom {
        let hex_ex_radius: MFloat = 1.0;
        let hex_in_radius = sqrt(
                pow(hex_ex_radius, 2) - pow(hex_ex_radius / 2.0, 2));
        Geom {
            hex_ex_radius: hex_ex_radius,
            hex_in_radius: hex_in_radius,
        }
    }

    pub fn map_pos_to_world_pos(&self, i: MapPos) -> WorldPos {
        let v = Vec2 {
            x: (i.x as MFloat) * self.hex_in_radius * 2.0,
            y: (i.y as MFloat) * self.hex_ex_radius * 1.5,
        };
        if i.y % 2 == 0 {
            Vec3{x: v.x + self.hex_in_radius, y: v.y, z: 0.0}
        } else {
            v.extend(0.0)
        }
    }

    pub fn index_to_circle_vertex(&self, count: MInt, i: MInt) -> VertexCoord {
        let n = FRAC_PI_2 + 2.0 * PI * (i as MFloat) / (count as MFloat);
        Vec3{x: cos(n), y: sin(n), z: 0.0}.mul_s(self.hex_ex_radius)
    }

    pub fn index_to_hex_vertex(&self, i: MInt) -> VertexCoord {
        self.index_to_circle_vertex(6, i)
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
