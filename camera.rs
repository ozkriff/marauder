// See LICENSE file for copyright and license details.

use std::num::{
  sin,
  cos
};
use cgmath::projection;
use cgmath::angle;
use cgmath::matrix::{
  Mat4
};
use cgmath::vector::{
  Vec3
};
use glh = gl_helpers;
use misc::deg_to_rad;

pub struct Camera {
  x_angle: f32,
  z_angle: f32,
  pos: Vec3<f32>,
  zoom: f32,
  projection_matrix: Mat4<f32>,
}

fn get_projection_matrix() -> Mat4<f32> {
  let fov = angle::deg(45.0f32);
  let ratio = 4.0 / 3.0;
  let display_range_min = 0.1;
  let display_range_max = 100.0;
  projection::perspective(
    fov, ratio, display_range_min, display_range_max
  )
}

impl Camera {
  pub fn new() -> Camera {
    Camera {
      x_angle: 45.0,
      z_angle: 0.0,
      pos: Vec3{x: 0.0, y: 0.0, z: 0.0},
      zoom: 10.0,
      projection_matrix: get_projection_matrix(),
    }
  }

  pub fn matrix(&self) -> Mat4<f32> {
    let mut mvp_matrix = self.projection_matrix;
    mvp_matrix = glh::tr(mvp_matrix, Vec3{x: 0.0f32, y: 0.0, z: -self.zoom});
    mvp_matrix = glh::rot_x(mvp_matrix, -self.x_angle);
    mvp_matrix = glh::rot_z(mvp_matrix, -self.z_angle);
    mvp_matrix = glh::tr(mvp_matrix, self.pos);
    mvp_matrix
  }

  pub fn move(&mut self, angle: f32) {
    let speed_in_radians = deg_to_rad(self.z_angle - angle);
    let dx = sin(speed_in_radians);
    let dy = cos(speed_in_radians);
    self.pos.x -= dy;
    self.pos.y -= dx;
  }
}


// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
