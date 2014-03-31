// See LICENSE file for copyright and license details.

use std;
use gl;
use gl::types::{GLuint, GLsizeiptr};
use cgmath::matrix::{Matrix, Mat4, Mat3, ToMat4};
use cgmath::vector::{Vec2, Vec3};
use cgmath::angle;
use core::misc::deg_to_rad;
use core::types::{Size2, MInt};
use visualizer::types::{MFloat};

pub use load_gl_funcs_with = gl::load_with;

pub enum MeshRenderMode {
    Triangles,
    Lines,
}

impl MeshRenderMode {
    fn to_gl_type(&self) -> GLuint {
        match *self {
            Triangles => gl::TRIANGLES,
            Lines => gl::LINES,
        }
    }
}

pub fn tr(m: Mat4<MFloat>, v: Vec3<MFloat>) -> Mat4<MFloat> {
    let mut t = Mat4::<MFloat>::identity();
    *t.mut_cr(3, 0) = v.x;
    *t.mut_cr(3, 1) = v.y;
    *t.mut_cr(3, 2) = v.z;
    m.mul_m(&t)
}

pub fn rot_x(m: Mat4<MFloat>, angle: MFloat) -> Mat4<MFloat> {
    let rad = angle::rad(deg_to_rad(angle));
    let r = Mat3::from_angle_x(rad).to_mat4();
    m.mul_m(&r)
}

pub fn rot_z(m: Mat4<MFloat>, angle: MFloat) -> Mat4<MFloat> {
    let rad = angle::rad(deg_to_rad(angle));
    let r = Mat3::from_angle_z(rad).to_mat4();
    m.mul_m(&r)
}

pub fn init_opengl() {
    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
}

pub fn set_clear_color(r: MFloat, g: MFloat, b: MFloat) {
    gl::ClearColor(r, g, b, 1.0);
}

pub fn clear_screen() {
    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
}

pub fn set_viewport(size: Size2<MInt>) {
    gl::Viewport(0, 0, size.w, size.h);
}

pub struct Vao {
    priv id: GLuint,
}

impl Vao {
    pub fn new() -> Vao {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        let vao = Vao{id: id};
        vao.bind();
        gl::EnableVertexAttribArray(id);
        vao
    }

    pub fn bind(&self) {
        gl::BindVertexArray(self.id);
    }

    pub fn unbind(&self) {
        gl::BindVertexArray(0);
    }

    pub fn draw_array(&self, mesh_mode: MeshRenderMode, faces_count: MInt) {
        let starting_index = 0;
        let vertices_count = faces_count * 3;
        let mode = mesh_mode.to_gl_type();
        gl::DrawArrays(mode, starting_index, vertices_count);
    }
}

impl Drop for Vao {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

pub struct Vbo {
    priv id: GLuint,
}

fn get_new_vbo_id() -> GLuint {
    let mut id = 0;
    unsafe {
        gl::GenBuffers(1, &mut id);
    }
    id
}

impl Vbo {
    pub fn from_data<T>(data: &[T]) -> Vbo {
        let vbo = Vbo{id: get_new_vbo_id()};
        vbo.bind();
        let size = std::mem::size_of::<T>();
        let buf_size = (data.len() * size) as GLsizeiptr;
        if data.len() != 0 {
            unsafe {
                let data_ptr = std::cast::transmute(&data[0]);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    buf_size,
                    data_ptr,
                    gl::STATIC_DRAW,
                );
            }
        }
        vbo
    }

    pub fn bind(&self) {
        gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
    }
}

impl Drop for Vbo {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

pub fn read_pixel_bytes(
    win_size: Size2<MInt>,
    mouse_pos: Vec2<MInt>,
) -> (MInt, MInt, MInt, MInt) {
    let height = win_size.h;
    let reverted_h = height - mouse_pos.y;
    let data: [u8, ..4] = [0, 0, 0, 0]; // mut
    unsafe {
        let data_ptr = std::cast::transmute(&data[0]);
        gl::ReadPixels(
            mouse_pos.x, reverted_h, 1, 1,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data_ptr
        );
    }
    (data[0] as MInt, data[1] as MInt, data[2] as MInt, data[3] as MInt)
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
