// See LICENSE file for copyright and license details.

use std;
use gl;
use gl::types::{
  GLfloat,
  GLint,
  GLuint,
};
use cgmath::vector::{
  Vec3,
  Vec2,
};
use glh = gl_helpers;
use map::TileIterator;
use color::Color3;
use camera::Camera;
use geom::Geom;

static PICK_VERTEX_SHADER_SRC: &'static str = "
  #version 130
  in vec3 position;
  in vec3 color;
  out vec3 pass_color;
  uniform mat4 mvp_mat;
  void main() {
    vec4 v = vec4(position, 1);
    gl_Position = mvp_mat * v;
    pass_color = color;
  }
";

static PICK_FRAGMENT_SHADER_SRC: &'static str = "
  #version 130
  in vec3 pass_color;
  out vec4 out_color;
  void main() {
    out_color = vec4(pass_color, 1.0);
  }
";

pub struct TilePicker {
  program: GLuint,
  color_buffer_obj: GLuint,
  mat_id: GLint,
  vertex_buffer_obj: GLuint,
  vertex_data: ~[Vec3<GLfloat>],
  color_data: ~[Color3],
}

impl TilePicker {
  pub fn new() -> TilePicker {
    let picker = TilePicker {
      color_buffer_obj: 0,
      program: 0,
      vertex_buffer_obj: 0,
      mat_id: 0,
      vertex_data: ~[],
      color_data: ~[],
    };
    picker
  }

  pub fn cleanup_opengl(&self) {
    gl::DeleteProgram(self.program);
    glh::delete_buffer(self.vertex_buffer_obj);
    glh::delete_buffer(self.color_buffer_obj);
  }

  fn build_hex_mesh_for_picking(&mut self, geom: &Geom) {
    for tile_pos in TileIterator::new() {
      let pos3d = geom.v2i_to_v2f(tile_pos).extend(0.0);
      for num in range(0, 6) {
        let vertex = geom.index_to_hex_vertex(num);
        let next_vertex = geom.index_to_hex_vertex(num + 1);
        let col_x = tile_pos.x as f32 / 255.0;
        let col_y = tile_pos.y as f32 / 255.0;
        let c_data = &mut self.color_data;
        let v_data = &mut self.vertex_data;
        let color = Color3{r: col_x, g: col_y, b: 1.0};
        v_data.push(pos3d + vertex.extend(0.0));
        c_data.push(color);
        v_data.push(pos3d + next_vertex.extend(0.0));
        c_data.push(color);
        v_data.push(pos3d + Vec3::zero());
        c_data.push(color);
      }
    }
  }

  pub fn init(&mut self, geom: &Geom) {
    self.build_hex_mesh_for_picking(geom);
    self.program = glh::compile_program(
      PICK_VERTEX_SHADER_SRC,
      PICK_FRAGMENT_SHADER_SRC);
    unsafe {
      gl::UseProgram(self.program);
      self.vertex_buffer_obj = glh::gen_buffer();
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_obj);
      glh::fill_current_coord_vbo(self.vertex_data);
      let pos_attr = glh::get_attr(self.program, "position");
      gl::EnableVertexAttribArray(pos_attr);
      glh::vertex_attrib_pointer(pos_attr);
      gl::GenBuffers(1, &mut self.color_buffer_obj);
      gl::BindBuffer(gl::ARRAY_BUFFER, self.color_buffer_obj);
      glh::fill_current_color_vbo(self.color_data);
      let color_attr = glh::get_attr(self.program, "color");
      gl::EnableVertexAttribArray(color_attr);
      glh::vertex_attrib_pointer(color_attr);
      self.mat_id = glh::get_uniform(self.program, "mvp_mat");
    }
  }

  fn _pick_tile(
    &self,
    win_size: (i32, i32),
    mouse_pos: Vec2<i32>
  ) -> Option<Vec2<i32>> {
    let (_, height) = win_size;
    let reverted_y = height - mouse_pos.y;
    let data: [u8, ..4] = [0, 0, 0, 0]; // mut
    unsafe {
      let data_ptr = std::cast::transmute(&data[0]);
      gl::ReadPixels(
        mouse_pos.x, reverted_y, 1, 1,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        data_ptr
      );
    }
    if data[2] != 0 {
      Some(Vec2{x: data[0] as i32, y: data[1] as i32})
    } else {
      None
    }
  }

  pub fn pick_tile(
    &mut self,
    win_size: (i32, i32),
    camera: &Camera,
    mouse_pos: Vec2<i32>
  ) -> Option<Vec2<i32>> {
    gl::UseProgram(self.program);
    gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_obj);
    gl::BindBuffer(gl::ARRAY_BUFFER, self.color_buffer_obj);
    glh::uniform_mat4f(self.mat_id, &camera.mat());
    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
    glh::draw_mesh(self.vertex_data);
    self._pick_tile(win_size, mouse_pos)
  }
}

impl Drop for TilePicker {
  fn drop(&mut self) {
    self.cleanup_opengl();
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
