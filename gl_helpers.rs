// See LICENSE file for copyright and license details.

use std;
use gl;
use gl::types::{
    GLuint,
    GLsizeiptr,
};
use cgmath::matrix::{
    Matrix,
    Mat4,
    Mat3,
    ToMat4,
};
use cgmath::vector::{
    Vec2,
    Vec3,
};
use cgmath::angle;
use misc::deg_to_rad;
use gl_types::{
    Float,
    MatId,
};
use core_types::{
    Size2,
    Int,
};

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

pub fn uniform_mat4f(mat_id: MatId, mat: &Mat4<Float>) {
    unsafe {
        let MatId(id) = mat_id;
        gl::UniformMatrix4fv(id as Int, 1, gl::FALSE, mat.cr(0, 0));
    }
}

pub fn tr(m: Mat4<Float>, v: Vec3<Float>) -> Mat4<Float> {
    let mut t = Mat4::<Float>::identity();
    *t.mut_cr(3, 0) = v.x;
    *t.mut_cr(3, 1) = v.y;
    *t.mut_cr(3, 2) = v.z;
    m.mul_m(&t)
}

pub fn rot_x(m: Mat4<Float>, angle: Float) -> Mat4<Float> {
    let rad = angle::rad(deg_to_rad(angle));
    let r = Mat3::from_angle_x(rad).to_mat4();
    m.mul_m(&r)
}

pub fn rot_z(m: Mat4<Float>, angle: Float) -> Mat4<Float> {
    let rad = angle::rad(deg_to_rad(angle));
    let r = Mat3::from_angle_z(rad).to_mat4();
    m.mul_m(&r)
}

pub fn init_opengl() {
    gl::Enable(gl::DEPTH_TEST);
}

pub fn set_clear_color(r: Float, g: Float, b: Float) {
    gl::ClearColor(r, g, b, 1.0);
}

pub fn clear_screen() {
    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
}

pub fn set_viewport(size: Size2<Int>) {
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

    pub fn draw_array(&self, mesh_mode: MeshRenderMode, faces_count: Int) {
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
        unsafe {
            let data_ptr = std::cast::transmute(&data[0]);
            gl::BufferData(gl::ARRAY_BUFFER, buf_size, data_ptr, gl::STATIC_DRAW);
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

pub fn get_vec2_from_pixel(
    win_size: Size2<Int>,
    mouse_pos: Vec2<Int>,
) -> Option<Vec2<Int>> {
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
    if data[2] != 0 {
        Some(Vec2{x: data[0] as Int, y: data[1] as Int})
    } else {
        None
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
