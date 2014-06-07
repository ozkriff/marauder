// See LICENSE file for copyright and license details.

use std::f32::consts::{PI, FRAC_PI_2};
use std::num::{pow, abs};
use cgmath::vector::{Vector2, Vector3, Vector};
use core::types::{MInt, MapPos};
use core::misc::{rad_to_deg};
use visualizer::types::{WorldPos, MFloat, VertexCoord};

pub static HEX_EX_RADIUS: MFloat = 1.0;
// (pow(1.0, 2) - pow(0.5, 2)).sqrt()
pub static HEX_IN_RADIUS: MFloat = 0.866025403784 * HEX_EX_RADIUS;

pub fn map_pos_to_world_pos(i: MapPos) -> WorldPos {
    let v = Vector2 {
        x: (i.v.x as MFloat) * HEX_IN_RADIUS * 2.0,
        y: (i.v.y as MFloat) * HEX_EX_RADIUS * 1.5,
    };
    if i.v.y % 2 == 0 {
        WorldPos{v: Vector3{
            x: v.x + HEX_IN_RADIUS,
            y: v.y,
            z: 0.0,
        }}
    } else {
        WorldPos{v: v.extend(0.0)}
    }
}

pub fn index_to_circle_vertex(count: MInt, i: MInt) -> VertexCoord {
    let n = FRAC_PI_2 + 2.0 * PI * (i as MFloat) / (count as MFloat);
    VertexCoord{
        v: Vector3{
            x: n.cos(),
            y: n.sin(),
            z: 0.0
        }.mul_s(HEX_EX_RADIUS)
    }
}

pub fn index_to_hex_vertex(i: MInt) -> VertexCoord {
    index_to_circle_vertex(6, i)
}

pub fn index_to_hex_vertex_s(scale: MFloat, i: MInt) -> VertexCoord {
    let v = index_to_hex_vertex(i).v.mul_s(scale);
    VertexCoord{v: v}
}

pub fn dist(a: WorldPos, b: WorldPos) -> MFloat {
    let dx = abs(b.v.x - a.v.x);
    let dy = abs(b.v.y - a.v.y);
    let dz = abs(b.v.z - a.v.z);
    (pow(dx, 2) + pow(dy, 2) + pow(dz, 2)).sqrt()
}

pub fn get_rot_angle(a: WorldPos, b: WorldPos) -> MFloat {
    let mut angle = rad_to_deg(((b.v.x - a.v.x) / dist(a, b)).asin());
    if b.v.y - a.v.y > 0.0 {
        angle = -(180.0 + angle);
    }
    angle
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
