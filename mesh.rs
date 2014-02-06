// See LICENSE file for copyright and license details.

use gl;
use gl::types::{
  GLfloat,
  GLuint,
};
use cgmath::vector::Vec3;
use glh = gl_helpers;

pub struct Mesh {
  vbo: GLuint,
  len: int,
}

impl Mesh {
  pub fn new() -> Mesh {
    Mesh {
      vbo: 0,
      len: 0,
    }
  }

  pub fn init(&mut self, data: &[Vec3<GLfloat>]) {
    self.len = data.len() as int;
    self.vbo = glh::gen_buffer();
    gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
    glh::fill_current_coord_vbo(data);
  }

  pub fn draw(&self, program: GLuint) {
    gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
    glh::vertex_attrib_pointer(glh::get_attr(program, "position"));
    glh::draw_mesh(self.len);
  }
}

impl Drop for Mesh {
  fn drop(&mut self) {
    glh::delete_buffer(self.vbo);
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
