// See LICENSE file for copyright and license details.

// Marauder is turn-based strategy game with hex grid.

extern mod glfw;
extern mod gl;
extern mod cgmath;
extern mod native;

use std::f32::consts::{
  PI,
  FRAC_PI_2
};
use std::num::{
  sqrt,
  pow,
  abs,
  sin,
  cos
};
use gltypes = gl::types;
use cgmath::matrix::{
  Matrix,
  Mat4,
  Mat3,
  ToMat4
};
use cgmath::vector::{
  Vec3,
  Vec2,
  Vector
};
use cgmath::projection;
use cgmath::angle;

#[link(name = "glfw")]
#[link(name = "m")]
extern {}

pub mod misc {
  use cgmath::matrix::Mat4;
  use std;

  pub fn print_mat4(name: &str, mat: Mat4<f32>) {
    println!("{}:", name);
    for i in range(0u, 4) {
      for j in range(0u, 4) {
        print!("{} ", *mat.cr(i, j));
      }
      println("");
    }
    println("");
  }

  /// usage: let s = mvp_matrix; println(type_of(&s));
  pub fn type_of<T>(_: &T) -> &'static str {
    unsafe {
      (*std::unstable::intrinsics::get_tydesc::<T>()).name
    }
  }
}

static mut MOUSE_POS: Vec2<f32> = Vec2{x: 0.0f32, y: 0.0};
static mut CAMERA_POS: Vec3<f32> = Vec3{x: 0.0f32, y: 0.0, z: 0.0};

static WIN_SIZE: Vec2<u32> = Vec2{x: 640, y: 480};
static VERTICES_COUNT: i32 = 3 * 2;

static VERTEX_DATA: [gltypes::GLfloat, ..VERTICES_COUNT * 3] = [
   0.0,  1.0, 0.0,
   2.0, -1.0, 0.0,
  -2.0, -1.0, 0.0,

  0.0,  1.0,  0.0,
  0.0, -1.0,  2.0,
  0.0, -1.0, -2.0
];

static VERTEX_SHADER_SRC: &'static str = "
  #version 130
  in vec3 position;
  uniform mat4 model_view_proj_matrix;
  void main() {
    vec4 v = vec4(position, 1);
    gl_Position = model_view_proj_matrix * v;
  }
";
 
static FRAGMENT_SHADER_SRC: &'static str = "
  #version 130
  out vec4 out_color;
  void main() {
    out_color = vec4(1.0, 1.0, 1.0, 1.0);
  }
";

pub struct Visualizer {
  hex_ex_radius: gltypes::GLfloat,
  hex_in_radius: gltypes::GLfloat
}

impl Visualizer {
  pub fn new() -> Visualizer {
    let hex_ex_radius: gltypes::GLfloat = 1.0 / 2.0;
    let hex_in_radius = sqrt(
        pow(hex_ex_radius, 2.0) - pow(hex_ex_radius / 2.0, 2.0));
    let visualizer = Visualizer {
      hex_ex_radius: hex_ex_radius,
      hex_in_radius: hex_in_radius
    };
    visualizer
  }

  pub fn dist(a: Vec2<f32>, b: Vec2<f32>) -> f32 {
    let dx = abs(b.x - a.x);
    let dy = abs(b.y - a.y);
    sqrt(pow(dx, 2.0) + pow(dy, 2.0))
  }

  pub fn v2i_to_v2f(&self, i: Vec2<i32>) -> Vec2<f32> {
    let v = Vec2 {
      x: (i.x as f32) * self.hex_in_radius * 2.0,
      y: (i.y as f32) * self.hex_ex_radius * 1.5
    };
    if i.y % 2 == 0 {
      Vec2{x: v.x + self.hex_in_radius, y: v.y}
    } else {
      v
    }
  }

  pub fn index_to_circle_vertex(&self, count: int, i: int) -> Vec2<f32> {
    let n = FRAC_PI_2 + 2.0 * PI * (i as f32) / (count as f32);
    Vec2{x: cos(n), y: sin(n)}.mul_s(self.hex_ex_radius)
  }

  pub fn index_to_hex_vertex(&self, i: int) -> Vec2<f32> {
    self.index_to_circle_vertex(6, i)
  }
}

fn compile_shader(src: &str, shader_type: gltypes::GLenum) -> gltypes::GLuint {
  let shader = gl::CreateShader(shader_type);
  unsafe {
    gl::ShaderSource(shader, 1, &src.to_c_str().unwrap(), std::ptr::null());
    gl::CompileShader(shader);

    let mut status = gl::FALSE as gltypes::GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    if status != (gl::TRUE as gltypes::GLint) {
      let mut len = 0;
      gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
      // subtract 1 to skip the trailing null character
      let mut buf = std::vec::from_elem(len as uint - 1, 0u8);
      gl::GetShaderInfoLog(shader, len, std::ptr::mut_null(),
        buf.as_mut_ptr() as *mut gltypes::GLchar
      );
      fail!("compile_shader(): " + std::str::raw::from_utf8(buf));
    }
  }
  shader
}

fn link_program(
    vertex_shader: gltypes::GLuint,
    fragment_shader: gltypes::GLuint
) -> gltypes::GLuint {
  let program = gl::CreateProgram();
  gl::AttachShader(program, vertex_shader);
  gl::AttachShader(program, fragment_shader);
  gl::LinkProgram(program);
  unsafe {
    let mut status = gl::FALSE as gltypes::GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    if status != (gl::TRUE as gltypes::GLint) {
      let mut len: gltypes::GLint = 0;
      gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
      // subtract 1 to skip the trailing null character
      let mut buf = std::vec::from_elem(len as uint - 1, 0u8);
      gl::GetProgramInfoLog(program, len, std::ptr::mut_null(),
        buf.as_mut_ptr() as *mut gltypes::GLchar
      );
      fail!("link_program(): " + std::str::raw::from_utf8(buf));
    }
  }
  program
}

fn tr(m: Mat4<f32>, v: Vec3<f32>) -> Mat4<f32> {
  let mut t = Mat4::<f32>::identity();
  *t.mut_cr(3, 0) = v.x;
  *t.mut_cr(3, 1) = v.y;
  *t.mut_cr(3, 2) = v.z;
  m.mul_m(&t)
}

fn rot_x(m: Mat4<f32>, angle: f32) -> Mat4<f32> {
  let r = Mat3::from_angle_x(angle::rad(angle)).to_mat4();
  m.mul_m(&r)
}

fn rot_y(m: Mat4<f32>, angle: f32) -> Mat4<f32> {
  let r = Mat3::from_angle_y(angle::rad(angle)).to_mat4();
  m.mul_m(&r)
}

struct Win {
  vertex_shader: gltypes::GLuint,
  fragment_shader: gltypes::GLuint,
  program: gltypes::GLuint,
  vertex_buffer_obj: gltypes::GLuint,
  matrix_id: gltypes::GLint,
  projection_matrix: Mat4<f32>,
  window: glfw::Window
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

impl Win {
  fn new() -> Win {
    let mut win = Win{
      vertex_shader: 0,
      fragment_shader: 0,
      program: 0,
      vertex_buffer_obj: 0,
      matrix_id: 0,
      projection_matrix: get_projection_matrix(),
      window: glfw::Window::create(
        WIN_SIZE.x,
        WIN_SIZE.y,
        "OpenGL",
        glfw::Windowed
      ).unwrap()
    };
    win.init_opengl();
    win.init_model();
    win
  }

  fn init_model(&mut self) {
    unsafe {
      // Create a Vertex Buffer Object
      gl::GenBuffers(1, &mut self.vertex_buffer_obj);
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_obj);

      // Copy vertex data to VBO
      let float_size = std::mem::size_of::<gltypes::GLfloat>();
      let vertices_ptr = (VERTEX_DATA.len() * float_size) as gltypes::GLsizeiptr;
      gl::BufferData(
        gl::ARRAY_BUFFER,
        vertices_ptr,
        std::cast::transmute(&VERTEX_DATA[0]),
        gl::STATIC_DRAW
      );

      gl::UseProgram(self.program);
      gl::BindFragDataLocation(
        self.program, 0, "out_color".to_c_str().unwrap());

      // Specify the layout of the vertex data
      let pos_attr = gl::GetAttribLocation(
        self.program, "position".to_c_str().unwrap()) as gltypes::GLuint;
      gl::EnableVertexAttribArray(pos_attr);

      let size = 3;
      let normalized = gl::FALSE;
      let stride = 0;
      gl::VertexAttribPointer(
        pos_attr,
        size,
        gl::FLOAT,
        normalized,
        stride,
        std::ptr::null()
      );

      self.matrix_id = gl::GetUniformLocation(
        self.program, "model_view_proj_matrix".to_c_str().unwrap()
      );
    }
  }

  fn init_opengl(&mut self) {
    // glfw::window_hint::context_version(3, 2);

    self.window.make_context_current();
    self.window.set_cursor_pos_callback(~CursorPosContext);
    self.window.set_key_callback(~KeyContext);

    // Load the OpenGL function pointers
    gl::load_with(glfw::get_proc_address);

    self.vertex_shader = compile_shader(
      VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
    self.fragment_shader = compile_shader(
      FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);
    self.program = link_program(self.vertex_shader, self.fragment_shader);
  }

  fn cleanup(&self) {
    gl::DeleteProgram(self.program);
    gl::DeleteShader(self.fragment_shader);
    gl::DeleteShader(self.vertex_shader);
    unsafe {
      gl::DeleteBuffers(1, &self.vertex_buffer_obj);
    }
  }

  fn update_matrices(&self) {
    let mut mvp_matrix = self.projection_matrix;
    unsafe {
      mvp_matrix = tr(mvp_matrix, Vec3{x: 0.0f32, y: 0.0, z: -10.0f32});
      mvp_matrix = rot_x(mvp_matrix, MOUSE_POS.y / 100.0);
      mvp_matrix = rot_y(mvp_matrix, MOUSE_POS.x / 100.0);
      mvp_matrix = tr(mvp_matrix, CAMERA_POS);

      // Send our transformation to the currently bound shader,
      // in the "model_view_proj_matrix" uniform for each model
      // you render, since the model_view_proj_matrix will be
      // different (at least the M part).
      gl::UniformMatrix4fv(self.matrix_id, 1, gl::FALSE, mvp_matrix.cr(0, 0));
    }
  }

  fn draw(&self) {
    self.update_matrices();
    gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
    gl::DrawArrays(gl::TRIANGLES, 0, VERTICES_COUNT);
    self.window.swap_buffers();
  }

  fn is_running(&self) -> bool {
    return !self.window.should_close()
  }

  fn process_events(&self) {
    glfw::poll_events();
  }
}

struct CursorPosContext;
impl glfw::CursorPosCallback for CursorPosContext {
  fn call(&self, w: &glfw::Window, xpos: f64, ypos: f64) {
    if w.get_mouse_button(glfw::MouseButtonRight) == glfw::Press {
      unsafe {
        MOUSE_POS.x = xpos as f32;
        MOUSE_POS.y = ypos as f32;
      }
    }
  }
}

struct ErrorContext;
impl glfw::ErrorCallback for ErrorContext {
  fn call(&self, _: glfw::Error, description: ~str) {
    println!("GLFW Error: {:s}", description);
  }
}

struct KeyContext;
impl glfw::KeyCallback for KeyContext {
  fn call(
    &self,
    window: &glfw::Window,
    key:    glfw::Key,
    _:      std::libc::c_int,
    action: glfw::Action,
    _:      glfw::Modifiers
  ) {
    let distance = 1.0;
    if action != glfw::Press {
      return;
    }
    match key {
      glfw::KeyEscape | glfw::KeyQ
                     => window.set_should_close(true),
      glfw::KeySpace => println!("space"),
      glfw::KeyUp    => unsafe { CAMERA_POS.y -= distance },
      glfw::KeyDown  => unsafe { CAMERA_POS.y += distance },
      glfw::KeyRight => unsafe { CAMERA_POS.x -= distance },
      glfw::KeyLeft  => unsafe { CAMERA_POS.x += distance },
      _ => {}
    }
  }
}

fn main() {
  glfw::set_error_callback(~ErrorContext);
  do glfw::start {
    let win =  Win::new();
    while win.is_running() {
      win.process_events();
      win.draw();
    }
    win.cleanup();
  }
}

#[start]
fn start(argc: int, argv: **u8) -> int {
  native::start(argc, argv, main)
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
