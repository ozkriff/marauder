// See LICENSE file for copyright and license details.

// Marauder is turn-based strategy game with hex grid.

extern mod glfw;
extern mod gl;
extern mod cgmath;

use std::comm::{
  Port,
  Chan,
  Data
};
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
use glt = gl::types;
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

struct KeyEvent {
  key: glfw::Key,
  action: glfw::Action
}

struct CursorPosEvent {
  x: f32,
  y: f32
}

static WIN_SIZE: Vec2<u32> = Vec2{x: 640, y: 480};

static VERTEX_SHADER_SRC: &'static str = "
  #version 130
  in vec3 position;
  uniform mat4 mvp_mat;
  void main() {
    vec4 v = vec4(position, 1);
    gl_Position = mvp_mat * v;
  }
";
 
static FRAGMENT_SHADER_SRC: &'static str = "
  #version 130
  out vec4 out_color;
  void main() {
    out_color = vec4(1.0, 1.0, 1.0, 1.0);
  }
";

struct Camera {
  x_angle: f32,
  z_angle: f32,
  pos: Vec3<f32>,
  zoom: f32,
  projection_matrix: Mat4<f32>,
}

impl Camera {
  pub fn new() -> Camera {
    Camera {
      x_angle: 45.0,
      z_angle: 0.0,
      pos: Vec3{x: 0.0, y: 0.0, z: 0.0},
      zoom: 10.0,
      projection_matrix: get_projection_matrix(),
    }
  }

  pub fn matrix(&self) -> Mat4<f32> {
    let mut mvp_matrix = self.projection_matrix;
    mvp_matrix = tr(mvp_matrix, Vec3{x: 0.0f32, y: 0.0, z: -self.zoom});
    mvp_matrix = rot_x(mvp_matrix, -self.x_angle);
    mvp_matrix = rot_z(mvp_matrix, -self.z_angle);
    mvp_matrix = tr(mvp_matrix, self.pos);
    mvp_matrix
  }

  pub fn move(&mut self, angle: f32) {
    // TODO: deg2rad, rename
    let speed_in_radians = (self.z_angle - angle) * PI / 180.0;
    let dx = sin(speed_in_radians);
    let dy = cos(speed_in_radians);
    self.pos.x -= dy;
    self.pos.y -= dx;
  }
}

pub struct Visualizer {
  hex_ex_radius: glt::GLfloat,
  hex_in_radius: glt::GLfloat
}

impl Visualizer {
  pub fn new() -> Visualizer {
    let hex_ex_radius: glt::GLfloat = 1.0 / 2.0;
    let hex_in_radius = sqrt(
        pow(hex_ex_radius, 2) - pow(hex_ex_radius / 2.0, 2));
    let visualizer = Visualizer {
      hex_ex_radius: hex_ex_radius,
      hex_in_radius: hex_in_radius
    };
    visualizer
  }

  pub fn dist(a: Vec2<f32>, b: Vec2<f32>) -> f32 {
    let dx = abs(b.x - a.x);
    let dy = abs(b.y - a.y);
    sqrt(pow(dx, 2) + pow(dy, 2))
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

fn compile_shader(src: &str, shader_type: glt::GLenum) -> glt::GLuint {
  let shader = gl::CreateShader(shader_type);
  unsafe {
    gl::ShaderSource(shader, 1, &src.to_c_str().unwrap(), std::ptr::null());
    gl::CompileShader(shader);

    let mut status = gl::FALSE as glt::GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    if status != (gl::TRUE as glt::GLint) {
      let mut len = 0;
      gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
      // subtract 1 to skip the trailing null character
      let mut buf = std::vec::from_elem(len as uint - 1, 0u8);
      gl::GetShaderInfoLog(shader, len, std::ptr::mut_null(),
        buf.as_mut_ptr() as *mut glt::GLchar
      );
      fail!("compile_shader(): " + std::str::raw::from_utf8(buf));
    }
  }
  shader
}

fn link_program(
    vertex_shader: glt::GLuint,
    fragment_shader: glt::GLuint
) -> glt::GLuint {
  let program = gl::CreateProgram();
  gl::AttachShader(program, vertex_shader);
  gl::AttachShader(program, fragment_shader);
  gl::LinkProgram(program);
  unsafe {
    let mut status = gl::FALSE as glt::GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

    if status != (gl::TRUE as glt::GLint) {
      let mut len: glt::GLint = 0;
      gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
      // subtract 1 to skip the trailing null character
      let mut buf = std::vec::from_elem(len as uint - 1, 0u8);
      gl::GetProgramInfoLog(program, len, std::ptr::mut_null(),
        buf.as_mut_ptr() as *mut glt::GLchar
      );
      fail!("link_program(): " + std::str::raw::from_utf8(buf));
    }
  }
  program
}

fn compile_program(
  vertex_shader_src: &str,
  frag_shader_src: &str
) -> glt::GLuint {
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

fn get_attr(program_id: glt::GLuint, name: &str) -> glt::GLuint {
  unsafe {
    let c_str = name.to_c_str().unwrap();
    gl::GetAttribLocation(program_id, c_str) as glt::GLuint
  }
}

fn get_uniform(program: glt::GLuint, name: &str) -> glt::GLint {
  unsafe {
    let c_str = name.to_c_str().unwrap();
    gl::GetUniformLocation(program, c_str)
  }
}

fn draw_mesh<T>(mesh: &[T]) {
  let starting_index = 0;
  let len = mesh.len() as i32;
  gl::DrawArrays(gl::TRIANGLES, starting_index, len);
}

pub fn uniform_mat4f(matrix_id: glt::GLint, matrix: &Mat4<glt::GLfloat>) {
  unsafe {
    gl::UniformMatrix4fv(matrix_id, 1, gl::FALSE, matrix.cr(0, 0));
  }
}

fn tr(m: Mat4<f32>, v: Vec3<f32>) -> Mat4<f32> {
  let mut t = Mat4::<f32>::identity();
  *t.mut_cr(3, 0) = v.x;
  *t.mut_cr(3, 1) = v.y;
  *t.mut_cr(3, 2) = v.z;
  m.mul_m(&t)
}

fn rot_x(m: Mat4<f32>, angle: f32) -> Mat4<f32> {
  let rad = angle::rad(angle * (PI / 180.0));
  let r = Mat3::from_angle_x(rad).to_mat4();
  m.mul_m(&r)
}

fn rot_z(m: Mat4<f32>, angle: f32) -> Mat4<f32> {
  let rad = angle::rad(angle * (PI / 180.0));
  let r = Mat3::from_angle_z(rad).to_mat4();
  m.mul_m(&r)
}

pub struct Win {
  key_event_port: Port<KeyEvent>,
  cursor_pos_event_port: Port<CursorPosEvent>,
  program: glt::GLuint,
  vertex_buffer_obj: glt::GLuint,
  matrix_id: glt::GLint,
  window: Option<glfw::Window>,
  vertex_data: ~[glt::GLfloat],
  mouse_pos: Vec2<f32>,
  camera: Camera,
  visualizer: Visualizer
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

// TODO: use iterator?
fn for_each_tile(f: |Vec2<i32>|) {
  let map_size = Vec2{x: 3, y: 4};
  for y in range(0i32, map_size.y) {
    for x in range(0i32, map_size.x) {
      f(Vec2{x: x, y: y});
    }
  }
}

fn fill_current_vbo(data: &[glt::GLfloat]) {
  let glfloat_size = std::mem::size_of::<glt::GLfloat>();
  let buffer_size = (data.len() * glfloat_size) as glt::GLsizeiptr;
  unsafe {
    gl::BufferData(
      gl::ARRAY_BUFFER,
      buffer_size,
      std::cast::transmute(&data[0]),
      gl::STATIC_DRAW
    );
  }
}

fn define_array_of_generic_attr_data(attr: glt::GLuint) {
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

fn add_point<T: Num>(
  vertex_data: &mut ~[T],
  pos: &Vec3<T>, x: T, y: T, z: T)
{
  vertex_data.push(x + pos.x);
  vertex_data.push(y + pos.y);
  vertex_data.push(z + pos.z);
}

fn handle_event_port<T: Send>(port: &Port<T>, f: |T|) {
  loop {
    match port.try_recv() {
      Data(e) => f(e),
      _ => break
    }
  }
}

impl Win {
  pub fn new() -> ~Win {
    let (key_event_port, key_event_chan) = Chan::new();
    let (cursor_pos_event_port, cursor_pos_chan) = Chan::new();
    let mut win = ~Win {
      key_event_port: key_event_port,
      cursor_pos_event_port: cursor_pos_event_port,
      program: 0,
      vertex_buffer_obj: 0,
      matrix_id: 0,
      window: None,
      vertex_data: ~[],
      mouse_pos: Vec2{x: 0.0f32, y: 0.0},
      camera: Camera::new(),
      visualizer: Visualizer::new()
    };
    win.init_glfw();
    win.init_opengl();
    win.init_model();
    win.window.get_ref().set_key_callback(
      ~KeyContext{chan: key_event_chan});
    win.window.get_ref().set_cursor_pos_callback(
      ~CursorPosContext{chan: cursor_pos_chan});
    win
  }

  fn build_hex_mesh(&mut self) {
    for_each_tile(|tile_pos| {
      let pos3d = self.visualizer.v2i_to_v2f(tile_pos).extend(0.0);
      for num in range(0, 6) {
        let vertex = self.visualizer.index_to_hex_vertex(num);
        let next_vertex = self.visualizer.index_to_hex_vertex(num + 1);
        let data = &mut self.vertex_data;
        add_point(data, &pos3d, vertex.x, vertex.y, 0.0);
        add_point(data, &pos3d, next_vertex.x, next_vertex.y, 0.0);
        add_point(data, &pos3d, 0.0, 0.0, 0.0);
      }
    });
  }

  fn init_model(&mut self) {
    self.build_hex_mesh();
    unsafe {
      gl::UseProgram(self.program);
      gl::GenBuffers(1, &mut self.vertex_buffer_obj);
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_obj);
      fill_current_vbo(self.vertex_data);
      let pos_attr = get_attr(self.program, "position");
      gl::EnableVertexAttribArray(pos_attr);
      define_array_of_generic_attr_data(pos_attr);
      self.matrix_id = get_uniform(self.program, "mvp_mat");
    }
  }

  fn init_glfw(&mut self) {
    // glfw::window_hint::context_version(3, 2);
    glfw::set_error_callback(~glfw::LogErrorHandler);
    glfw::init();
    self.window = Some(
      glfw::Window::create(
        WIN_SIZE.x,
        WIN_SIZE.y,
        "OpenGL",
        glfw::Windowed
      ).unwrap()
    );
    let window = self.window.get_ref();
    window.make_context_current();
  }

  fn init_opengl(&mut self) {
    gl::load_with(glfw::get_proc_address);
    self.program = compile_program(
      VERTEX_SHADER_SRC,
      FRAGMENT_SHADER_SRC);
  }

  pub fn cleanup_opengl(&self) {
    gl::DeleteProgram(self.program);
    unsafe {
      gl::DeleteBuffers(1, &self.vertex_buffer_obj);
    }
  }

  pub fn draw(&self) {
    gl::UseProgram(self.program);
    gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_obj);
    uniform_mat4f(self.matrix_id, &self.camera.matrix());
    gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
    draw_mesh(self.vertex_data);
    self.window.get_ref().swap_buffers();
  }

  pub fn is_running(&self) -> bool {
    return !self.window.get_ref().should_close()
  }

  pub fn process_events(&mut self) {
    let win = self.window.get_ref();
    glfw::poll_events();
    handle_event_port(&self.key_event_port, |e| {
      if e.action != glfw::Press {
        return;
      }
      match e.key {
        glfw::KeyEscape | glfw::KeyQ
                       => win.set_should_close(true),
        glfw::KeySpace => println!("space"),
        glfw::KeyUp    => self.camera.move(270.0),
        glfw::KeyDown  => self.camera.move(90.0),
        glfw::KeyRight => self.camera.move(0.0),
        glfw::KeyLeft  => self.camera.move(180.0),
        _ => {}
      }
    });
    handle_event_port(&self.cursor_pos_event_port, |e| {
      if win.get_mouse_button(glfw::MouseButtonRight) == glfw::Press {
        let dx = self.mouse_pos.x - e.x;
        let dy = self.mouse_pos.y - e.y;
        self.camera.z_angle += dx;
        self.camera.x_angle += dy;
      }
      self.mouse_pos = Vec2{x: e.x, y: e.y};
    });
  }

  fn close_window(&mut self) {
    // destroy glfw::Window before terminating glfw
    self.window = None;
  }
}

impl Drop for Win {
  fn drop(&mut self) {
    self.cleanup_opengl();
    self.close_window();
    glfw::terminate();
  }
}

struct CursorPosContext { chan: Chan<CursorPosEvent> }
impl glfw::CursorPosCallback for CursorPosContext {
  fn call(&self, _: &glfw::Window, xpos: f64, ypos: f64) {
    self.chan.send(CursorPosEvent {
      x: xpos as f32,
      y: ypos as f32
    });
  }
}

struct KeyContext { chan: Chan<KeyEvent> }
impl glfw::KeyCallback for KeyContext {
  fn call(
    &self,
    _:      &glfw::Window,
    key:    glfw::Key,
    _:      std::libc::c_int,
    action: glfw::Action,
    _:      glfw::Modifiers
  ) {
    self.chan.send(KeyEvent {
      key: key,
      action: action
    });
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
