// See LICENSE file for copyright and license details.

use std;
use gl;
use gl::types::{
  GLfloat,
  GLint,
  GLuint,
  GLchar,
  GLenum,
  GLsizeiptr,
  GLsizei,
};
use cgmath::matrix::{
  Matrix,
  Mat4,
  Mat3,
  ToMat4,
};
use cgmath::vector::{
  Vec3,
  Vec2,
};
use cgmath::angle;
use stb_image::image;
use misc::{
  c_str,
  deg_to_rad,
};
use color::Color3;

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
      let mut len = 0;
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
  frag_shader_src: &str,
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

pub fn draw_mesh(faces_count: int) {
  let starting_index = 0;
  let vertices_count = faces_count as i32 * 3;
  gl::DrawArrays(gl::TRIANGLES, starting_index, vertices_count);
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

pub fn gen_buffer() -> GLuint {
  let mut n = 0 as GLuint;
  unsafe {
    gl::GenBuffers(1, &mut n);
  }
  n
}

pub fn delete_buffer(buffer: GLuint) {
  unsafe {
    gl::DeleteBuffers(1, &buffer);
  }
}

fn fill_buffer<T>(buffer_size: i64, data: &[T]) {
  unsafe {
    let data_ptr = std::cast::transmute(&data[0]);
    gl::BufferData(gl::ARRAY_BUFFER, buffer_size, data_ptr, gl::STATIC_DRAW);
  }
}

pub fn fill_current_coord_vbo(data: &[Vec3<GLfloat>]) {
  let glfloat_size = std::mem::size_of::<GLfloat>();
  let buffer_size = (data.len() * 3 * glfloat_size) as GLsizeiptr;
  fill_buffer(buffer_size, data);
}

pub fn fill_current_color_vbo(data: &[Color3]) {
  let color3_size = std::mem::size_of::<Color3>();
  let buffer_size = (data.len() * color3_size) as GLsizeiptr;
  fill_buffer(buffer_size, data);
}

pub fn fill_current_texture_coords_vbo(data: &[Vec2<GLfloat>]) {
  let glfloat_size = std::mem::size_of::<GLfloat>();
  let buffer_size = (data.len() * 2 * glfloat_size) as GLsizeiptr;
  fill_buffer(buffer_size, data);
}

pub fn vertex_attrib_pointer(attr: GLuint, components_count: i32) {
  let normalized = gl::FALSE;
  let stride = 0;
  unsafe {
    gl::VertexAttribPointer(
      attr,
      components_count,
      gl::FLOAT,
      normalized,
      stride,
      std::ptr::null(),
    );
  }
}

fn load_image(path: ~str) -> image::Image<u8> {
  let load_result = image::load(path);
  match load_result {
    image::ImageU8(image) => {
      image
    },
    image::Error(message) => {
      fail!("{}", message);
    },
    _ => {
      fail!("unkn");
    }
  }
}

pub fn load_texture(path: ~str) -> GLuint {
  let image = load_image(path);
  let mut id = 0;
  unsafe {
    gl::GenTextures(1, &mut id)
  };
  gl::ActiveTexture(gl::TEXTURE0);
  gl::BindTexture(gl::TEXTURE_2D, id);
  let format = match image.depth {
    4 => gl::RGBA,
    3 => gl::RGB,
    _ => fail!("wrong depth"),
  };
  unsafe {
    let level = 0;
    let border = 0;
    gl::TexImage2D(
      gl::TEXTURE_2D,
      level,
      format as GLint,
      image.width as GLsizei,
      image.height as GLsizei,
      border,
      format,
      gl::UNSIGNED_BYTE,
      std::cast::transmute(&image.data[0]),
    );
  }
  gl::TexParameteri(gl::TEXTURE_2D,
    gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
  gl::TexParameteri(gl::TEXTURE_2D,
    gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
  gl::TexParameteri(gl::TEXTURE_2D,
    gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
  gl::TexParameteri(gl::TEXTURE_2D,
    gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
  id
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
