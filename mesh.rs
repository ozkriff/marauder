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
  vbo: GLuint,
  color_vbo: Option<GLuint>,
  vt_vbo: Option<GLuint>,
  len: int,
}

impl Mesh {
  pub fn new() -> Mesh {
    Mesh {
      vbo: 0,
      color_vbo: None,
      vt_vbo: None,
      len: 0,
    }
  }

  pub fn set_vertex_coords(&mut self, data: &[Vec3<GLfloat>]) {
    self.len = data.len() as int;
    self.vbo = glh::gen_buffer();
    gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
    glh::fill_current_coord_vbo(data);
  }

  pub fn set_color(&mut self, data: &[Color3]) {
    self.len = data.len() as int;
    self.color_vbo = Some(glh::gen_buffer());
    gl::BindBuffer(gl::ARRAY_BUFFER, self.color_vbo.unwrap());
    glh::fill_current_color_vbo(data);
  }

  pub fn set_vt(&mut self, data: &[Vec2<GLfloat>]) {
    self.len = data.len() as int;
    self.vt_vbo = Some(glh::gen_buffer());
    gl::BindBuffer(gl::ARRAY_BUFFER, self.vt_vbo.unwrap());
    glh::fill_current_vt_vbo(data);
  }

  pub fn draw(&self, program: GLuint) {
    if !self.vt_vbo.is_none() {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vt_vbo.unwrap());
      glh::vertex_attrib_pointer(glh::get_attr(program, "vt"), 2);
    }
    if !self.color_vbo.is_none() {
      gl::BindBuffer(gl::ARRAY_BUFFER, self.color_vbo.unwrap());
      glh::vertex_attrib_pointer(glh::get_attr(program, "color"), 3);
    }
    gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
    glh::vertex_attrib_pointer(glh::get_attr(program, "position"), 3);
    glh::draw_mesh(self.len);
  }
}

impl Drop for Mesh {
  fn drop(&mut self) {
    glh::delete_buffer(self.vbo);
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
