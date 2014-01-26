// See LICENSE file for copyright and license details.

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
  sin,
  cos
};
use glfw;
use gl;
use std;
use gl::types::{
  GLfloat,
  GLint,
  GLuint
};
use cgmath::matrix::{
  Matrix
};
use cgmath::vector::{
  Vec3,
  Vec2,
  Vector
};
use glh = gl_helpers;
use camera::Camera;

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

struct TilePicker {
  program: GLuint,
  color_buffer_obj: GLuint,
  mat_id: GLint,
  vertex_buffer_obj: GLuint,
  vertex_data: ~[GLfloat],
  color_data: ~[GLfloat],
  selected_tile_pos: Option<Vec2<int>>
}

impl TilePicker {
  fn new() -> TilePicker {
    TilePicker {
      color_buffer_obj: 0,
      program: 0,
      vertex_buffer_obj: 0,
      mat_id: 0,
      vertex_data: ~[],
      color_data: ~[],
      selected_tile_pos: None
    }
  }

  fn init_opengl(&mut self) {
    self.program = glh::compile_program(
      PICK_VERTEX_SHADER_SRC,
      PICK_FRAGMENT_SHADER_SRC);
  }

  pub fn cleanup_opengl(&self) {
    gl::DeleteProgram(self.program);
    unsafe {
      gl::DeleteBuffers(1, &self.vertex_buffer_obj);
      gl::DeleteBuffers(1, &self.color_buffer_obj);
    }
  }
}

impl Drop for TilePicker {
  fn drop(&mut self) {
    self.cleanup_opengl();
  }
}

pub struct Visualizer {
  hex_ex_radius: GLfloat,
  hex_in_radius: GLfloat,
  key_event_port: Port<KeyEvent>,
  cursor_pos_event_port: Port<CursorPosEvent>,
  program: GLuint,
  vertex_buffer_obj: GLuint,
  mat_id: GLint,
  glfw_win: Option<glfw::Window>,
  vertex_data: ~[GLfloat],
  mouse_pos: Vec2<f32>,
  camera: Camera,
  picker: TilePicker
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

fn add_point<T: Num>(
  vertex_data: &mut ~[T],
  pos: &Vec3<T>, x: T, y: T, z: T)
{
  vertex_data.push(x + pos.x);
  vertex_data.push(y + pos.y);
  vertex_data.push(z + pos.z);
}

fn add_color<T>(color_data: &mut ~[T], r: T, g: T, b: T) {
  color_data.push(r);
  color_data.push(g);
  color_data.push(b);
}

fn handle_event_port<T: Send>(port: &Port<T>, f: |T|) {
  loop {
    match port.try_recv() {
      Data(e) => f(e),
      _ => break
    }
  }
}

impl Visualizer {
  pub fn new() -> ~Visualizer {
    let hex_ex_radius: GLfloat = 1.0 / 2.0;
    let hex_in_radius = sqrt(
        pow(hex_ex_radius, 2) - pow(hex_ex_radius / 2.0, 2));
    let (key_event_port, key_event_chan) = Chan::new();
    let (cursor_pos_event_port, cursor_pos_chan) = Chan::new();
    let mut vis = ~Visualizer {
      hex_ex_radius: hex_ex_radius,
      hex_in_radius: hex_in_radius,
      key_event_port: key_event_port,
      cursor_pos_event_port: cursor_pos_event_port,
      program: 0,
      vertex_buffer_obj: 0,
      mat_id: 0,
      glfw_win: None,
      vertex_data: ~[],
      mouse_pos: Vec2{x: 0.0f32, y: 0.0},
      camera: Camera::new(),
      picker: TilePicker::new()
    };
    vis.init_glfw();
    vis.init_opengl();
    vis.init_model();
    vis.init_tile_picker();
    vis.glfw_win().set_key_callback(
      ~KeyContext{chan: key_event_chan});
    vis.glfw_win().set_cursor_pos_callback(
      ~CursorPosContext{chan: cursor_pos_chan});
    vis
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


  fn glfw_win<'a>(&'a self) -> &'a glfw::Window {
    self.glfw_win.get_ref()
  }

  fn build_hex_mesh(&mut self) {
    for_each_tile(|tile_pos| {
      let pos3d = self.v2i_to_v2f(tile_pos).extend(0.0);
      for num in range(0, 6) {
        let vertex = self.index_to_hex_vertex(num);
        let next_vertex = self.index_to_hex_vertex(num + 1);
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
      glh::fill_current_vbo(self.vertex_data);
      let pos_attr = glh::get_attr(self.program, "position");
      gl::EnableVertexAttribArray(pos_attr);
      glh::define_array_of_generic_attr_data(pos_attr);
      self.mat_id = glh::get_uniform(self.program, "mvp_mat");
    }
  }

  fn init_glfw(&mut self) {
    // glfw::window_hint::context_version(3, 2);
    glfw::set_error_callback(~glfw::LogErrorHandler);
    glfw::init();
    self.glfw_win = Some(
      glfw::Window::create(
        WIN_SIZE.x,
        WIN_SIZE.y,
        "OpenGL",
        glfw::Windowed
      ).unwrap()
    );
    self.glfw_win().make_context_current();
  }

  fn init_opengl(&mut self) {
    gl::load_with(glfw::get_proc_address);
    self.program = glh::compile_program(
      VERTEX_SHADER_SRC,
      FRAGMENT_SHADER_SRC);
    self.picker.init_opengl();
  }

  pub fn cleanup_opengl(&self) {
    gl::DeleteProgram(self.program);
    unsafe {
      gl::DeleteBuffers(1, &self.vertex_buffer_obj);
    }
    self.picker.cleanup_opengl();
  }

  fn draw_map(&self) {
    gl::UseProgram(self.program);
    gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_obj);
    glh::uniform_mat4f(self.mat_id, &self.camera.mat());
    glh::draw_mesh(self.vertex_data);
  }

  pub fn draw(&self) {
    gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
    self.draw_map();
    self.glfw_win().swap_buffers();
  }

  pub fn is_running(&self) -> bool {
    return !self.glfw_win().should_close()
  }

  pub fn process_events(&mut self) {
    glfw::poll_events();
    handle_event_port(&self.key_event_port, |e| {
      if e.action != glfw::Press {
        return;
      }
      match e.key {
        glfw::KeyEscape | glfw::KeyQ
                       => self.glfw_win().set_should_close(true),
        glfw::KeySpace => println!("space"),
        glfw::KeyUp    => self.camera.move(270.0),
        glfw::KeyDown  => self.camera.move(90.0),
        glfw::KeyRight => self.camera.move(0.0),
        glfw::KeyLeft  => self.camera.move(180.0),
        _ => {}
      }
    });
    handle_event_port(&self.cursor_pos_event_port, |e| {
      let button = self.glfw_win().get_mouse_button(glfw::MouseButtonRight);
      if button == glfw::Press {
        self.camera.z_angle += self.mouse_pos.x - e.x;
        self.camera.x_angle += self.mouse_pos.y - e.y;
      }
      self.mouse_pos = Vec2{x: e.x, y: e.y};
    });
  }

  fn close_window(&mut self) {
    // destroy glfw::Window before terminating glfw
    self.glfw_win = None;
  }

  fn build_hex_mesh_for_picking(&mut self) {
    for_each_tile(|tile_pos| {
      let pos3d = self.v2i_to_v2f(tile_pos).extend(0.0);
      for num in range(0, 6) {
        let vertex = self.index_to_hex_vertex(num);
        let next_vertex = self.index_to_hex_vertex(num + 1);
        let col_x = tile_pos.x as f32 / 255.0;
        let col_y = tile_pos.y as f32 / 255.0;
        let c_data = &mut self.picker.color_data;
        let v_data = &mut self.picker.vertex_data;
        add_point(v_data, &pos3d, vertex.x, vertex.y, 0.0);
        add_color(c_data, col_x, col_y, 1.0);
        add_point(v_data, &pos3d, next_vertex.x, next_vertex.y, 0.0);
        add_color(c_data, col_x, col_y, 1.0);
        add_point(v_data, &pos3d, 0.0, 0.0, 0.0);
        add_color(c_data, col_x, col_y, 1.0);
      }
    });
  }

  fn init_tile_picker(&mut self) {
    self.build_hex_mesh_for_picking();
    unsafe {
      gl::UseProgram(self.picker.program);
      gl::GenBuffers(1, &mut self.picker.vertex_buffer_obj);
      gl::BindBuffer(gl::ARRAY_BUFFER, self.picker.vertex_buffer_obj);
      glh::fill_current_vbo(self.picker.vertex_data);
      let pos_attr = glh::get_attr(self.picker.program, "position");
      gl::EnableVertexAttribArray(pos_attr);
      glh::define_array_of_generic_attr_data(pos_attr);
      gl::GenBuffers(1, &mut self.picker.color_buffer_obj);
      gl::BindBuffer(gl::ARRAY_BUFFER, self.picker.color_buffer_obj);
      glh::fill_current_vbo(self.picker.color_data);
      let color_attr = glh::get_attr(self.picker.program, "color");
      gl::EnableVertexAttribArray(color_attr);
      glh::define_array_of_generic_attr_data(color_attr);
      self.picker.mat_id = glh::get_uniform(self.picker.program, "mvp_mat");
    }
  }

  fn _pick_tile(&self, x: i32, y: i32) -> Option<Vec2<int>> {
    let (_, height) = self.glfw_win().get_size();
    let reverted_y = height - y;
    let data: [u8, ..4] = [0, 0, 0, 0]; // mut
    unsafe {
      let data_ptr = std::cast::transmute(&data[0]);
      gl::ReadPixels(
        x, reverted_y, 1, 1,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        data_ptr
      );
    }
    if data[2] != 0 {
      Some(Vec2{x: data[0] as int, y: data[1] as int})
    } else {
      None
    }
  }

  pub fn pick_tile(&mut self) {
    gl::UseProgram(self.picker.program);
    gl::BindBuffer(gl::ARRAY_BUFFER, self.picker.vertex_buffer_obj);
    gl::BindBuffer(gl::ARRAY_BUFFER, self.picker.color_buffer_obj);
    glh::uniform_mat4f(self.picker.mat_id, &self.camera.mat());
    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
    glh::draw_mesh(self.picker.vertex_data);
    self.picker.selected_tile_pos = self._pick_tile(
      self.mouse_pos.x as i32,
      self.mouse_pos.y as i32
    );
    println!("selected: {:?}", self.picker.selected_tile_pos);
  }
}

impl Drop for Visualizer {
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
