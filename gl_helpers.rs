// See LICENSE file for copyright and license details.

use std;
use gl;
use gl::types::{
  GLfloat,
  GLint,
  GLuint,
  GLchar,
  GLenum
};
use cgmath::matrix::{
  Matrix,
  Mat4,
  Mat3,
  ToMat4
};
use cgmath::vector::Vec3;
use cgmath::angle;
use misc::{
  c_str,
  deg_to_rad
};

pub fn compile_shader(src: &str, shader_type: GLenum) -> GLuint {
  let shader = gl::CreateShader(shader_type);
  unsafe {
    gl::ShaderSource(shader, 1, &c_str(src), std::ptr::null());
    gl::CompileShader(shader);
    let mut status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
    if status != (gl::TRUE as GLint) {
      let mut len = 0;
      gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
      // subtract 1 to skip the trailing null character
      let mut buf = std::vec::from_elem(len as uint - 1, 0u8);
      gl::GetShaderInfoLog(shader, len, std::ptr::mut_null(),
        buf.as_mut_ptr() as *mut GLchar
      );
      fail!("compile_shader(): " + std::str::raw::from_utf8(buf));
    }
  }
  shader
}

pub fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
  let program = gl::CreateProgram();
  gl::AttachShader(program, vertex_shader);
  gl::AttachShader(program, fragment_shader);
  gl::LinkProgram(program);
  unsafe {
    let mut status = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
    if status != (gl::TRUE as GLint) {
      let mut len: GLint = 0;
      gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
      // subtract 1 to skip the trailing null character
      let mut buf = std::vec::from_elem(len as uint - 1, 0u8);
      gl::GetProgramInfoLog(program, len, std::ptr::mut_null(),
        buf.as_mut_ptr() as *mut GLchar
      );
      fail!("link_program(): " + std::str::raw::from_utf8(buf));
    }
  }
  program
}

pub fn compile_program(
  vertex_shader_src: &str,
  frag_shader_src: &str
) -> GLuint {
  let vertex_shader = compile_shader(
    vertex_shader_src, gl::VERTEX_SHADER);
  let fragment_shader = compile_shader(
    frag_shader_src, gl::FRAGMENT_SHADER);
  let program = link_program(vertex_shader, fragment_shader);
  // mark shaders for deletion after program deletion
  gl::DeleteShader(fragment_shader);
  gl::DeleteShader(vertex_shader);
  program
}

pub fn get_attr(program_id: GLuint, name: &str) -> GLuint {
  unsafe {
    gl::GetAttribLocation(program_id, c_str(name)) as GLuint
  }
}

pub fn get_uniform(program: GLuint, name: &str) -> GLint {
  unsafe {
    gl::GetUniformLocation(program, c_str(name))
  }
}

pub fn draw_mesh<T>(mesh: &[T]) {
  let starting_index = 0;
  let len = mesh.len() as i32 * 3;
  gl::DrawArrays(gl::TRIANGLES, starting_index, len);
}

pub fn uniform_mat4f(mat_id: GLint, mat: &Mat4<GLfloat>) {
  unsafe {
    gl::UniformMatrix4fv(mat_id, 1, gl::FALSE, mat.cr(0, 0));
  }
}

pub fn tr(m: Mat4<f32>, v: Vec3<f32>) -> Mat4<f32> {
  let mut t = Mat4::<f32>::identity();
  *t.mut_cr(3, 0) = v.x;
  *t.mut_cr(3, 1) = v.y;
  *t.mut_cr(3, 2) = v.z;
  m.mul_m(&t)
}

pub fn rot_x(m: Mat4<f32>, angle: f32) -> Mat4<f32> {
  let rad = angle::rad(deg_to_rad(angle));
  let r = Mat3::from_angle_x(rad).to_mat4();
  m.mul_m(&r)
}

pub fn rot_z(m: Mat4<f32>, angle: f32) -> Mat4<f32> {
  let rad = angle::rad(deg_to_rad(angle));
  let r = Mat3::from_angle_z(rad).to_mat4();
  m.mul_m(&r)
}

pub fn fill_current_coord_vbo(data: &[Vec3<GLfloat>]) {
  let glfloat_size = std::mem::size_of::<GLfloat>();
  let buffer_size = (data.len() * 3 * glfloat_size) as gl::types::GLsizeiptr;
  unsafe {
    gl::BufferData(
      gl::ARRAY_BUFFER,
      buffer_size,
      std::cast::transmute(&data[0]),
      gl::STATIC_DRAW
    );
  }
}

pub fn fill_current_color_vbo(data: &[Vec3<GLfloat>]) {
  fill_current_coord_vbo(data); // May change later
}

pub fn define_array_of_generic_attr_data(attr: GLuint) {
  let components_count = 3;
  let normalized = gl::FALSE;
  let stride = 0;
  unsafe {
    gl::VertexAttribPointer(
      attr,
      components_count,
      gl::FLOAT,
      normalized,
      stride,
      std::ptr::null()
    );
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
