// See LICENSE file for copyright and license details.

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
use glfw_events::EventHandlers;
use map::TileIterator;

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
  vertex_data: ~[Vec3<GLfloat>],
  color_data: ~[Vec3<GLfloat>],
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
  glfw_event_handlers: EventHandlers,
  program: GLuint,
  vertex_buffer_obj: GLuint,
  mat_id: GLint,
  win: Option<glfw::Window>,
  vertex_data: ~[Vec3<GLfloat>],
  mouse_pos: Vec2<f32>,
  camera: Camera,
  picker: TilePicker
}

fn init_win() -> glfw::Window {
  glfw::set_error_callback(~glfw::LogErrorHandler);
  glfw::init();
  let win = glfw::Window::create(
    WIN_SIZE.x,
    WIN_SIZE.y,
    "OpenGL",
    glfw::Windowed
  ).unwrap();
  win.make_context_current();
  win
}

impl Visualizer {
  pub fn new() -> ~Visualizer {
    let hex_ex_radius: GLfloat = 1.0 / 2.0;
    let hex_in_radius = sqrt(
        pow(hex_ex_radius, 2) - pow(hex_ex_radius / 2.0, 2));
    let win = init_win();
    let mut vis = ~Visualizer {
      hex_ex_radius: hex_ex_radius,
      hex_in_radius: hex_in_radius,
      glfw_event_handlers: EventHandlers::new(&win),
      program: 0,
      vertex_buffer_obj: 0,
      mat_id: 0,
      win: Some(win),
      vertex_data: ~[],
      mouse_pos: Vec2::zero(),
      camera: Camera::new(),
      picker: TilePicker::new()
    };
    vis.init_opengl();
    vis.init_model();
    vis.init_tile_picker();
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


  fn win<'a>(&'a self) -> &'a glfw::Window {
    self.win.get_ref()
  }

  fn build_hex_mesh(&mut self) {
    for tile_pos in TileIterator::new() {
      let pos3d = self.v2i_to_v2f(tile_pos).extend(0.0);
      for num in range(0, 6) {
        let vertex = self.index_to_hex_vertex(num);
        let next_vertex = self.index_to_hex_vertex(num + 1);
        let data = &mut self.vertex_data;
        data.push(pos3d + vertex.extend(0.0));
        data.push(pos3d + next_vertex.extend(0.0));
        data.push(pos3d + Vec3::zero());
      }
    }
  }

  fn init_model(&mut self) {
    self.build_hex_mesh();
    unsafe {
      gl::UseProgram(self.program);
      gl::GenBuffers(1, &mut self.vertex_buffer_obj);
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_obj);
      glh::fill_current_coord_vbo(self.vertex_data);
      let pos_attr = glh::get_attr(self.program, "position");
      gl::EnableVertexAttribArray(pos_attr);
      glh::define_array_of_generic_attr_data(pos_attr);
      self.mat_id = glh::get_uniform(self.program, "mvp_mat");
    }
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
    self.win().swap_buffers();
  }

  pub fn is_running(&self) -> bool {
    return !self.win().should_close()
  }

  pub fn handle_key_events(&mut self) {
    self.glfw_event_handlers.key_handler.handle(|event| {
      if event.action != glfw::Press {
        return;
      }
      match event.key {
        glfw::KeyEscape | glfw::KeyQ
                       => self.win().set_should_close(true),
        glfw::KeySpace => println!("space"),
        glfw::KeyUp    => self.camera.move(270.0),
        glfw::KeyDown  => self.camera.move(90.0),
        glfw::KeyRight => self.camera.move(0.0),
        glfw::KeyLeft  => self.camera.move(180.0),
        _ => {}
      }
    });
  }

  pub fn handle_cursor_pos_events(&mut self) {
    self.glfw_event_handlers.cursor_pos_handler.handle(|event| {
      let button = self.win().get_mouse_button(glfw::MouseButtonRight);
      if button == glfw::Press {
        self.camera.z_angle += self.mouse_pos.x - event.x;
        self.camera.x_angle += self.mouse_pos.y - event.y;
      }
      self.mouse_pos = Vec2{x: event.x, y: event.y};
    });
  }

  pub fn handle_events(&mut self) {
    glfw::poll_events();
    self.handle_key_events();
    self.handle_cursor_pos_events();
  }

  fn close_window(&mut self) {
    // destroy glfw::Window before terminating glfw
    self.win = None;
  }

  fn build_hex_mesh_for_picking(&mut self) {
    for tile_pos in TileIterator::new() {
      let pos3d = self.v2i_to_v2f(tile_pos).extend(0.0);
      for num in range(0, 6) {
        let vertex = self.index_to_hex_vertex(num);
        let next_vertex = self.index_to_hex_vertex(num + 1);
        let col_x = tile_pos.x as f32 / 255.0;
        let col_y = tile_pos.y as f32 / 255.0;
        let c_data = &mut self.picker.color_data;
        let v_data = &mut self.picker.vertex_data;
        let color = Vec3{x: col_x, y: col_y, z: 1.0};
        v_data.push(pos3d + vertex.extend(0.0));
        c_data.push(color);
        v_data.push(pos3d + next_vertex.extend(0.0));
        c_data.push(color);
        v_data.push(pos3d + Vec3::zero());
        c_data.push(color);
      }
    }
  }

  fn init_tile_picker(&mut self) {
    self.build_hex_mesh_for_picking();
    unsafe {
      gl::UseProgram(self.picker.program);
      gl::GenBuffers(1, &mut self.picker.vertex_buffer_obj);
      gl::BindBuffer(gl::ARRAY_BUFFER, self.picker.vertex_buffer_obj);
      glh::fill_current_coord_vbo(self.picker.vertex_data);
      let pos_attr = glh::get_attr(self.picker.program, "position");
      gl::EnableVertexAttribArray(pos_attr);
      glh::define_array_of_generic_attr_data(pos_attr);
      gl::GenBuffers(1, &mut self.picker.color_buffer_obj);
      gl::BindBuffer(gl::ARRAY_BUFFER, self.picker.color_buffer_obj);
      glh::fill_current_color_vbo(self.picker.color_data);
      let color_attr = glh::get_attr(self.picker.program, "color");
      gl::EnableVertexAttribArray(color_attr);
      glh::define_array_of_generic_attr_data(color_attr);
      self.picker.mat_id = glh::get_uniform(self.picker.program, "mvp_mat");
    }
  }

  fn _pick_tile(&self, x: i32, y: i32) -> Option<Vec2<int>> {
    let (_, height) = self.win().get_size();
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

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
