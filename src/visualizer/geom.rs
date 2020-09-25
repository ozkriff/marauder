// See LICENSE file for copyright and license details.

use crate::core::misc::rad_to_deg;
use crate::core::types::{MInt, MapPos};
use crate::visualizer::types::{MFloat, VertexCoord, WorldPos};
use cgmath::{Vector, Vector2, Vector3};
use std::f32::consts::{FRAC_PI_2, PI};

pub const HEX_EX_RADIUS: MFloat = 1.0;
// (pow(1.0, 2) - pow(0.5, 2)).sqrt()
pub const HEX_IN_RADIUS: MFloat = 0.866025403784 * HEX_EX_RADIUS;

pub const MINIMAL_LIFT_HEIGHT: MFloat = 0.01;

pub fn lift(v: Vector3<MFloat>) -> Vector3<MFloat> {
    let mut v = v;
    v.z += MINIMAL_LIFT_HEIGHT;
    v
}

pub fn map_pos_to_world_pos(i: MapPos) -> WorldPos {
    let v = Vector2 {
        x: (i.v.x as MFloat) * HEX_IN_RADIUS * 2.0,
        y: (i.v.y as MFloat) * HEX_EX_RADIUS * 1.5,
    };
    if i.v.y % 2 == 0 {
        WorldPos {
            v: Vector3 {
                x: v.x + HEX_IN_RADIUS,
                y: v.y,
                z: 0.0,
            },
        }
    } else {
        WorldPos { v: v.extend(0.0) }
    }
}

pub fn index_to_circle_vertex(count: MInt, i: MInt) -> VertexCoord {
    let n = FRAC_PI_2 + 2.0 * PI * (i as MFloat) / (count as MFloat);
    VertexCoord {
        v: Vector3 {
            x: n.cos(),
            y: n.sin(),
            z: 0.0,
        }
        .mul_s(HEX_EX_RADIUS),
    }
}

pub fn index_to_hex_vertex(i: MInt) -> VertexCoord {
    index_to_circle_vertex(6, i)
}

pub fn index_to_hex_vertex_s(scale: MFloat, i: MInt) -> VertexCoord {
    let v = index_to_hex_vertex(i).v.mul_s(scale);
    VertexCoord { v }
}

pub fn dist(a: WorldPos, b: WorldPos) -> MFloat {
    let dx = ((b.v.x as f32).abs() - (a.v.x as f32).abs()).abs();
    let dy = ((b.v.y as f32).abs() - (a.v.y as f32).abs()).abs();
    let dz = ((b.v.z as f32).abs() - (a.v.z as f32).abs()).abs();

    (dx.powf(2.0) + dy.powf(2.0) + dz.powf(2.0)).sqrt()
}

pub fn get_rot_angle(a: WorldPos, b: WorldPos) -> MFloat {
    let mut angle = rad_to_deg(((b.v.x - a.v.x) / dist(a, b)).asin());
    if b.v.y - a.v.y > 0.0 {
        angle = -(180.0 + angle);
    }
    angle
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
