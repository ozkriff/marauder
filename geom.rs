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
use core::MapPos;
use core_types::{
  Int,
};
use gl_types::{
  WorldPos,
  Float,
  VertexCoord,
};

pub struct Geom {
  hex_ex_radius: Float,
  hex_in_radius: Float,
}

impl Geom {
  pub fn new() -> Geom {
    let hex_ex_radius: Float = 1.0 / 2.0;
    let hex_in_radius = sqrt(
        pow(hex_ex_radius, 2) - pow(hex_ex_radius / 2.0, 2));
    Geom {
      hex_ex_radius: hex_ex_radius,
      hex_in_radius: hex_in_radius,
    }
  }

  pub fn map_pos_to_world_pos(&self, i: MapPos) -> WorldPos {
    let v = Vec2 {
      x: (i.x as Float) * self.hex_in_radius * 2.0,
      y: (i.y as Float) * self.hex_ex_radius * 1.5,
    };
    if i.y % 2 == 0 {
      Vec3{x: v.x + self.hex_in_radius, y: v.y, z: 0.0}
    } else {
      v.extend(0.0)
    }
  }

  pub fn index_to_circle_vertex(&self, count: Int, i: Int) -> VertexCoord {
    let n = FRAC_PI_2 + 2.0 * PI * (i as Float) / (count as Float);
    Vec3{x: cos(n), y: sin(n), z: 0.0}.mul_s(self.hex_ex_radius)
  }

  pub fn index_to_hex_vertex(&self, i: Int) -> VertexCoord {
    self.index_to_circle_vertex(6, i)
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
