// See LICENSE file for copyright and license details.

use cgmath::projection;
use cgmath::angle;
use cgmath::matrix::Mat4;
use cgmath::vector::Vec3;
use core::types::{MInt, Size2};
use core::misc::deg_to_rad;
use visualizer::gl_helpers::{tr, rot_x, rot_z};
use visualizer::types::{MFloat, WorldPos};

pub struct Camera {
    pub x_angle: MFloat,
    pub z_angle: MFloat,
    pub pos: WorldPos,
    pub zoom: MFloat,
    projection_mat: Mat4<MFloat>,
}

fn get_projection_mat(win_size: Size2<MInt>) -> Mat4<MFloat> {
    let fov = angle::deg(45.0 as MFloat);
    let ratio = win_size.w as MFloat / win_size.h as MFloat;
    let display_range_min = 0.1;
    let display_range_max = 100.0;
    projection::perspective(
        fov, ratio, display_range_min, display_range_max)
}

impl Camera {
    pub fn new(win_size: Size2<MInt>) -> Camera {
        Camera {
            x_angle: 45.0,
            z_angle: 0.0,
            pos: Vec3::zero(),
            zoom: 10.0,
            projection_mat: get_projection_mat(win_size),
        }
    }

    pub fn mat(&self) -> Mat4<MFloat> {
        let mut m = self.projection_mat;
        m = tr(m, Vec3{x: 0.0, y: 0.0, z: -self.zoom});
        m = rot_x(m, -self.x_angle);
        m = rot_z(m, -self.z_angle);
        m = tr(m, self.pos);
        m
    }

    pub fn move(&mut self, angle: MFloat, speed: MFloat) {
        let speed_in_radians = deg_to_rad(self.z_angle - angle);
        let dx = speed_in_radians.sin();
        let dy = speed_in_radians.cos();
        self.pos.x -= dy * speed * self.zoom;
        self.pos.y -= dx * speed * self.zoom;
    }

    pub fn set_win_size(&mut self, win_size: Size2<MInt>) {
        self.projection_mat = get_projection_mat(win_size);
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
