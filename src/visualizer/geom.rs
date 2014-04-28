// See LICENSE file for copyright and license details.

use std::f32::consts::{PI, FRAC_PI_2};
use std::num::{pow, abs};
use cgmath::vector::{Vector2, Vector3, Vector};
use core::types::{MInt, MapPos, SlotId};
use core::misc::{rad_to_deg};
use core::core::SLOTS_COUNT;
use visualizer::types::{WorldPos, MFloat, VertexCoord};

pub struct Geom {
    pub hex_ex_radius: MFloat,
    pub hex_in_radius: MFloat,
}

impl Geom {
    pub fn new() -> Geom {
        let hex_ex_radius: MFloat = 1.2;
        let hex_in_radius =
            (pow(hex_ex_radius, 2) - pow(hex_ex_radius / 2.0, 2)).sqrt();
        Geom {
            hex_ex_radius: hex_ex_radius,
            hex_in_radius: hex_in_radius,
        }
    }

    pub fn map_pos_to_world_pos(&self, i: MapPos) -> WorldPos {
        let v = Vector2 {
            x: (i.v.x as MFloat) * self.hex_in_radius * 2.0,
            y: (i.v.y as MFloat) * self.hex_ex_radius * 1.5,
        };
        if i.v.y % 2 == 0 {
            Vector3{x: v.x + self.hex_in_radius, y: v.y, z: 0.0}
        } else {
            v.extend(0.0)
        }
    }

    pub fn index_to_circle_vertex(
        &self,
        count: MInt,
        i: MInt
    ) -> VertexCoord {
        let n = FRAC_PI_2 + 2.0 * PI * (i as MFloat) / (count as MFloat);
        Vector3{x: n.cos(), y: n.sin(), z: 0.0}.mul_s(self.hex_ex_radius)
    }

    pub fn index_to_hex_vertex(&self, i: MInt) -> VertexCoord {
        self.index_to_circle_vertex(6, i)
    }

    pub fn slot_pos(&self, slot_index: SlotId) -> VertexCoord {
        self.index_to_circle_vertex(SLOTS_COUNT, slot_index.id).mul_s(0.6)
    }

    pub fn dist(&self, a: WorldPos, b: WorldPos) -> MFloat {
        let dx = abs(b.x - a.x);
        let dy = abs(b.y - a.y);
        let dz = abs(b.z - a.z);
        (pow(dx, 2) + pow(dy, 2) + pow(dz, 2)).sqrt()
    }

    pub fn get_rot_angle(&self, a: WorldPos, b: WorldPos) -> MFloat {
        let mut angle = rad_to_deg(((b.x - a.x) / self.dist(a, b)).asin());
        if b.y - a.y > 0.0 {
            angle = -(180.0 + angle);
        }
        angle
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
