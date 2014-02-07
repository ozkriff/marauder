// See LICENSE file for copyright and license details.

use glfw;
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
use camera::Camera;
use map::TileIterator;
use geom::Geom;
use tile_picker::TilePicker;
use obj;
use mesh::Mesh;
use misc::read_file;

pub struct Visualizer {
  program: GLuint,
  map_mesh: Mesh,
  unit_mesh: Mesh,
  mat_id: GLint,
  win: Option<glfw::Window>,
  mouse_pos: Vec2<f32>,
  camera: Camera,
  picker: TilePicker,
  selected_tile_pos: Option<Vec2<i32>>,
  geom: Geom,
}

fn init_win(win_size: Vec2<int>) -> glfw::Window {
  glfw::set_error_callback(~glfw::LogErrorHandler);
  let init_status = glfw::init();
  if !init_status.is_ok() {
    fail!("Failed to initialize GLFW");
  }
  let win = glfw::Window::create(
    win_size.x as u32,
    win_size.y as u32,
    "OpenGL",
    glfw::Windowed,
  ).unwrap();
  win.make_context_current();
  win.set_cursor_pos_polling(true);
  win.set_key_polling(true);
  win
}

impl Visualizer {
  pub fn new() -> ~Visualizer {
    let win_size = Vec2::<int>{x: 640, y: 480};
    let win = init_win(win_size);
    let geom = Geom::new();
    let mut vis = ~Visualizer {
      program: 0,
      map_mesh: Mesh::new(),
      unit_mesh: Mesh::new(),
      mat_id: 0,
      win: Some(win),
      mouse_pos: Vec2::zero(),
      camera: Camera::new(),
      picker: TilePicker::new(),
      selected_tile_pos: None,
      geom: geom,
    };
    vis.init_opengl();
    vis.picker.init(&geom);
    vis.init_models();
    vis
  }

  fn win<'a>(&'a self) -> &'a glfw::Window {
    self.win.get_ref()
  }

  fn build_hex_mesh(&self) -> ~[Vec3<GLfloat>] {
    let mut vertex_data = ~[];
    for tile_pos in TileIterator::new() {
      let pos3d = self.geom.v2i_to_v2f(tile_pos).extend(0.0);
      for num in range(0, 6) {
        let vertex = self.geom.index_to_hex_vertex(num);
        let next_vertex = self.geom.index_to_hex_vertex(num + 1);
        let data = &mut vertex_data;
        data.push(pos3d + vertex.extend(0.0));
        data.push(pos3d + next_vertex.extend(0.0));
        data.push(pos3d + Vec3::zero());
      }
    }
    vertex_data
  }

  fn init_models(&mut self) {
    self.program = glh::compile_program(
      read_file(&Path::new("normal.vs.glsl")),
      read_file(&Path::new("normal.fs.glsl")),
    );
    gl::UseProgram(self.program);
    self.mat_id = glh::get_uniform(self.program, "mvp_mat");
    let pos_attr = glh::get_attr(self.program, "position");
    glh::vertex_attrib_pointer(pos_attr);
    let map_vertex_data = self.build_hex_mesh();
    self.map_mesh.init(map_vertex_data);
    let unit_obj = obj::Model::new("tank.obj");
    self.unit_mesh.init(unit_obj.build());
  }

  fn init_opengl(&mut self) {
    gl::load_with(glfw::get_proc_address);
  }

  fn cleanup_opengl(&self) {
    gl::DeleteProgram(self.program);
  }

  fn draw(&self) {
    gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
    gl::UseProgram(self.program);
    glh::uniform_mat4f(self.mat_id, &self.camera.mat());
    self.map_mesh.draw(self.program);
    self.unit_mesh.draw(self.program);
    self.win().swap_buffers();
  }

  pub fn is_running(&self) -> bool {
    return !self.win().should_close()
  }

  fn handle_key_event(&mut self, key: glfw::Key) {
    match key {
      glfw::KeyEscape | glfw::KeyQ => {
        self.win().set_should_close(true);
      },
      glfw::KeySpace => println!("space"),
      glfw::KeyUp => self.camera.move(270.0),
      glfw::KeyDown => self.camera.move(90.0),
      glfw::KeyRight => self.camera.move(0.0),
      glfw::KeyLeft => self.camera.move(180.0),
      _ => {},
    }
  }

  fn handle_cursor_pos_event(&mut self, pos: Vec2<f32>) {
    let button = self.win().get_mouse_button(glfw::MouseButtonRight);
    if button == glfw::Press {
      self.camera.z_angle += self.mouse_pos.x - pos.x;
      self.camera.x_angle += self.mouse_pos.y - pos.y;
    }
    self.mouse_pos = pos;
  }

  fn get_events(&mut self) -> ~[glfw::WindowEvent]{
    glfw::poll_events();
    let mut events = ~[];
    for (_, event) in self.win().flush_events() {
      events.push(event);
    }
    events
  }

  fn handle_event(&mut self, event: glfw::WindowEvent) {
    match event {
      glfw::KeyEvent(key, _, glfw::Press, _) => {
        self.handle_key_event(key);
      },
      glfw::CursorPosEvent(x, y) => {
        let p = Vec2{x: x as f32, y: y as f32};
        self.handle_cursor_pos_event(p);
      },
      _ => {},
    }
  }

  fn handle_events(&mut self) {
    for event in self.get_events().iter() {
      self.handle_event(*event);
    }
  }

  fn close_window(&mut self) {
    // destroy glfw::Window before terminating glfw
    self.win = None;
  }

  fn pick_tile(&mut self) {
    let mouse_pos = Vec2 {
      x: self.mouse_pos.x as i32,
      y: self.mouse_pos.y as i32,
    };
    let win_size = self.win().get_size();
    self.selected_tile_pos = self.picker.pick_tile(
      win_size, &self.camera, mouse_pos);
    println!("selected: {:?}", self.selected_tile_pos);
  }

  pub fn tick(&mut self) {
    self.handle_events();
    self.pick_tile();
    self.draw();
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
