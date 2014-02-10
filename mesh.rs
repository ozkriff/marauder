// See LICENSE file for copyright and license details.

use gl;
use gl::types::{
  GLfloat,
  GLuint,
};
use cgmath::vector::{
  Vec3,
  Vec2,
};
use glh = gl_helpers;
use color::Color3;

pub struct Mesh {
  vertex_coords_vbo: GLuint,
  color_vbo: Option<GLuint>,
  texture_coords_vbo: Option<GLuint>,
  length: int,
}

impl Mesh {
  pub fn new() -> Mesh {
    Mesh {
      vertex_coords_vbo: 0,
      color_vbo: None,
      texture_coords_vbo: None,
      length: 0,
    }
  }

  pub fn set_vertex_coords(&mut self, data: &[Vec3<GLfloat>]) {
    self.length = data.len() as int;
    self.vertex_coords_vbo = glh::gen_buffer();
    gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_coords_vbo);
    glh::fill_current_coord_vbo(data);
  }

  pub fn set_color(&mut self, data: &[Color3]) {
    self.length = data.len() as int;
    self.color_vbo = Some(glh::gen_buffer());
    gl::BindBuffer(gl::ARRAY_BUFFER, self.color_vbo.unwrap());
    glh::fill_current_color_vbo(data);
  }

  pub fn set_texture_coords(&mut self, data: &[Vec2<GLfloat>]) {
    self.length = data.len() as int;
    self.texture_coords_vbo = Some(glh::gen_buffer());
    gl::BindBuffer(gl::ARRAY_BUFFER, self.texture_coords_vbo.unwrap());
    glh::fill_current_texture_coords_vbo(data);
  }

  pub fn draw(&self, program: GLuint) {
    if !self.texture_coords_vbo.is_none() {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.texture_coords_vbo.unwrap());
      glh::vertex_attrib_pointer(glh::get_attr(program, "vt"), 2);
    }
    if !self.color_vbo.is_none() {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.color_vbo.unwrap());
      glh::vertex_attrib_pointer(glh::get_attr(program, "color"), 3);
    }
    gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_coords_vbo);
    glh::vertex_attrib_pointer(glh::get_attr(program, "position"), 3);
    glh::draw_mesh(self.length);
  }
}

impl Drop for Mesh {
  fn drop(&mut self) {
    glh::delete_buffer(self.vertex_coords_vbo);
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
