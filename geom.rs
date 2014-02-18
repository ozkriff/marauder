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
use gl::types::GLfloat;
use cgmath::vector::{
  Vec2,
  Vec3,
  Vector,
};
use core::MapPos;
use event_visualizer::WorldPos;

pub struct Geom {
  hex_ex_radius: GLfloat,
  hex_in_radius: GLfloat,
}

impl Geom {
  pub fn new() -> Geom {
    let hex_ex_radius: GLfloat = 1.0 / 2.0;
    let hex_in_radius = sqrt(
        pow(hex_ex_radius, 2) - pow(hex_ex_radius / 2.0, 2));
    Geom {
      hex_ex_radius: hex_ex_radius,
      hex_in_radius: hex_in_radius,
    }
  }

  pub fn map_pos_to_world_pos(&self, i: MapPos) -> WorldPos {
    let v = Vec2 {
      x: (i.x as f32) * self.hex_in_radius * 2.0,
      y: (i.y as f32) * self.hex_ex_radius * 1.5,
    };
    if i.y % 2 == 0 {
      Vec3{x: v.x + self.hex_in_radius, y: v.y, z: 0.0}
    } else {
      v.extend(0.0)
    }
  }

  pub fn index_to_circle_vertex(&self, count: int, i: int) -> Vec3<f32> {
    let n = FRAC_PI_2 + 2.0 * PI * (i as f32) / (count as f32);
    Vec3{x: cos(n), y: sin(n), z: 0.0}.mul_s(self.hex_ex_radius)
  }

  pub fn index_to_hex_vertex(&self, i: int) -> Vec3<f32> {
    self.index_to_circle_vertex(6, i)
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
