// See LICENSE file for copyright and license details.

use extra::json;
use std::hashmap::HashMap;
use serialize::Decodable;
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

fn build_hex_mesh(&geom: &Geom) -> ~[Vec3<GLfloat>] {
  let mut vertex_data = ~[];
  for tile_pos in TileIterator::new() {
    let pos = geom.map_pos_to_world_pos(tile_pos);
    for num in range(0, 6) {
      let vertex = geom.index_to_hex_vertex(num);
      let next_vertex = geom.index_to_hex_vertex(num + 1);
      vertex_data.push(pos + vertex);
      vertex_data.push(pos + next_vertex);
      vertex_data.push(pos + Vec3::zero());
    }
  }
  vertex_data
}

fn build_hex_tex_coord() -> ~[Vec2<GLfloat>] {
  let mut vertex_data = ~[];
  for _ in TileIterator::new() {
    for _ in range(0, 6) {
      vertex_data.push(Vec2{x: 0.0, y: 0.0});
      vertex_data.push(Vec2{x: 1.0, y: 0.0});
      vertex_data.push(Vec2{x: 0.5, y: 0.5});
    }
  }
  vertex_data
}

pub struct SceneNode {
  pos: Vec3<f32>,
}

#[deriving(Decodable)]
struct Size2<T> {
  x: T,
  y: T,
}

fn read_win_size(config_path: &str) -> Vec2<i32> {
  let path = Path::new(config_path);
  let json = json::from_str(read_file(&path)).unwrap();
  let mut decoder = json::Decoder::new(json);
  let size: Size2<i32> = Decodable::decode(&mut decoder);
  Vec2{x: size.x, y: size.y}
}

fn init_win(win_size: Vec2<i32>) -> glfw::Window {
  glfw::set_error_callback(~glfw::LogErrorHandler);
  let init_status = glfw::init();
  if !init_status.is_ok() {
    fail!("Failed to initialize GLFW");
  }
  let win = glfw::Window::create(
    win_size.x as u32,
    win_size.y as u32,
    "Marauder",
    glfw::Windowed,
  ).unwrap();
  win.make_context_current();
  win.set_all_polling(true);
  win
}

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
  scene_nodes: HashMap<i32, SceneNode>,
}

impl Visualizer {
  pub fn new() -> ~Visualizer {
    let win_size = read_win_size("config.json");
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
      picker: TilePicker::new(win_size),
      selected_tile_pos: None,
      geom: geom,
      scene_nodes: HashMap::new(),
    };
    vis.init_opengl();
    vis.picker.init(&geom);
    vis.init_models();
    vis.init_units();
    vis
  }

  fn win<'a>(&'a self) -> &'a glfw::Window {
    self.win.get_ref()
  }

  fn init_models(&mut self) {
    self.program = glh::compile_program(
      read_file(&Path::new("normal.vs.glsl")),
      read_file(&Path::new("normal.fs.glsl")),
    );
    gl::UseProgram(self.program);
    self.mat_id = glh::get_uniform(self.program, "mvp_mat");
    let vertex_coordinates_attr = glh::get_attr(
      self.program, "in_vertex_coordinates");
    gl::EnableVertexAttribArray(vertex_coordinates_attr);
    glh::vertex_attrib_pointer(vertex_coordinates_attr, 3);
    let texture_coords_attr = glh::get_attr(
      self.program, "in_texture_coordinates");
    gl::EnableVertexAttribArray(texture_coords_attr);
    glh::vertex_attrib_pointer(texture_coords_attr, 3);
    let map_vertex_data = build_hex_mesh(&self.geom);
    self.map_mesh.set_vertex_coords(map_vertex_data);
    self.map_mesh.set_texture_coords(build_hex_tex_coord());
    self.map_mesh.set_texture(glh::load_texture(~"data/floor.png"));
    let unit_obj = obj::Model::new("data/tank.obj");
    self.unit_mesh.set_vertex_coords(unit_obj.build());
    self.unit_mesh.set_texture_coords(unit_obj.build_tex_coord());
    self.unit_mesh.set_texture(glh::load_texture(~"data/tank.png"));
  }

  fn add_unit(&mut self, id: i32, pos: Vec2<i32>) {
    let world_pos = self.geom.map_pos_to_world_pos(pos);
    self.scene_nodes.insert(id, SceneNode{pos: world_pos});
  }

  fn init_units(&mut self) {
    self.add_unit(0, Vec2{x: 0, y: 0});
    self.add_unit(1, Vec2{x: 0, y: 1});
    self.add_unit(2, Vec2{x: 1, y: 0});
    self.add_unit(2, Vec2{x: 1, y: 1});
  }

  fn init_opengl(&mut self) {
    gl::load_with(glfw::get_proc_address);
    gl::Enable(gl::DEPTH_TEST);
  }

  fn cleanup_opengl(&self) {
    gl::DeleteProgram(self.program);
  }

  fn draw_units(&self) {
    gl::UseProgram(self.program);
    for (_, unit) in self.scene_nodes.iter() {
      let m = glh::tr(self.camera.mat(), unit.pos);
      glh::uniform_mat4f(self.mat_id, &m);
      self.unit_mesh.draw(self.program);
    }
  }

  fn draw_map(&self) {
    glh::uniform_mat4f(self.mat_id, &self.camera.mat());
    self.map_mesh.draw(self.program);
  }

  fn draw(&self) {
    gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    gl::UseProgram(self.program);
    self.draw_units();
    self.draw_map();
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
      glfw::KeyMinus => self.camera.zoom += 1.0,
      glfw::KeyEqual => self.camera.zoom -= 1.0,
      _ => {},
    }
  }

  fn handle_cursor_pos_event(&mut self, pos: Vec2<f32>) {
    let button = self.win().get_mouse_button(glfw::MouseButtonRight);
    if button == glfw::Press {
      self.camera.z_angle += (self.mouse_pos.x - pos.x) / 2.0;
      self.camera.x_angle += (self.mouse_pos.y - pos.y) / 2.0;
    }
    self.mouse_pos = pos;
  }

  fn handle_mouse_button_event(&mut self) {
    if !self.selected_tile_pos.is_none() {
      let map_pos = self.selected_tile_pos.unwrap();
      let pos = self.geom.map_pos_to_world_pos(map_pos);
      let unit_id = 0;
      self.scene_nodes.get_mut(&unit_id).pos = pos;
    }
  }

  fn get_events(&mut self) -> ~[glfw::WindowEvent] {
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
      glfw::MouseButtonEvent(glfw::MouseButtonLeft, glfw::Press, _) => {
        self.handle_mouse_button_event();
      },
      glfw::SizeEvent(w, h) => {
        gl::Viewport(0, 0, w, h);
        self.picker.set_win_size(Vec2{x: w, y: h});
      },
      _ => {},
    }
  }

  fn handle_events(&mut self) {
    for event in self.get_events().iter() {
      self.handle_event(*event);
    }
  }

  fn pick_tile(&mut self) {
    let mouse_pos = Vec2 {
      x: self.mouse_pos.x as i32,
      y: self.mouse_pos.y as i32,
    };
    self.selected_tile_pos = self.picker.pick_tile(&self.camera, mouse_pos);
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
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
