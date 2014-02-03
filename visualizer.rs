// See LICENSE file for copyright and license details.

use std::io::{
  BufferedReader,
  File
};
use std::str::Words;
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
use glfw_events::EventHandlers;
use map::TileIterator;
use geom::Geom;
use tile_picker::TilePicker;

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

struct ObjFace {
  vertex: [int, ..3],
  texture: [int, ..3],
  normal: [int, ..3],
}

struct ObjModel {
  coords: ~[Vec3<GLfloat>],
  normals: ~[Vec3<GLfloat>],
  texture_coords: ~[Vec2<GLfloat>],
  faces: ~[ObjFace],
}

// TODO: unwrap() -> ...
impl ObjModel {
  fn new(filename: &str) -> ObjModel {
    let mut obj = ObjModel {
      coords: ~[],
      normals: ~[],
      texture_coords: ~[],
      faces: ~[],
    };
    obj.read(filename);
    obj
  }

  fn read_v_or_vn(words: &mut Words) -> Vec3<GLfloat> {
    Vec3 {
      x: from_str(words.next().unwrap()).unwrap(),
      y: from_str(words.next().unwrap()).unwrap(),
      z: from_str(words.next().unwrap()).unwrap(),
    }
  }

  fn read_vt(words: &mut Words) -> Vec2<GLfloat> {
    // TODO: y = 1.0 - y; // flip vertically
    Vec2 {
      x: from_str(words.next().unwrap()).unwrap(),
      y: from_str(words.next().unwrap()).unwrap(),
    }
  }

  fn read_f(words: &mut Words) -> ObjFace {
    let mut face = ObjFace {
      vertex: [0, 0, 0],
      texture: [0, 0, 0],
      normal: [0, 0, 0],
    };
    let mut i = 0;
    for group in *words {
      let mut w = group.split('/');
      face.vertex[i] = from_str(w.next().unwrap()).unwrap();
      face.texture[i] = from_str(w.next().unwrap()).unwrap();
      face.normal[i] = from_str(w.next().unwrap()).unwrap();
      i += 1;
    }
    face
  }

  fn read(&mut self, filename: &str) {
    let path = Path::new(filename);
    let mut file = BufferedReader::new(File::open(&path));
    for line in file.lines() {
      let mut words = line.words(); // TODO: Remove mut
      fn is_correct_tag(tag: &str) -> bool {
        tag.len() != 0 && tag[0] != ('#' as u8)
      }
      let tag = match words.next() {
        Some(w) if is_correct_tag(w) => w,
        _ => "", // TODO: ???
      };
      match tag {
        &"v" => self.coords.push(ObjModel::read_v_or_vn(&mut words)),
        &"vn" => self.normals.push(ObjModel::read_v_or_vn(&mut words)),
        &"vt" => self.texture_coords.push(ObjModel::read_vt(&mut words)),
        &"f" => self.faces.push(ObjModel::read_f(&mut words)),
        _ => print!("."),
      }
    }
  }

  fn build(&self) -> ~[Vec3<GLfloat>]{
    let mut mesh = ~[];
    for face in self.faces.iter() {
      for i in range(0, 3) {
        let vertex_id = face.vertex[i] - 1;
        // let texture_coord_id = face.texture[i] - 1; // TODO
        mesh.push(self.coords[vertex_id]);
      }
    }
    mesh
  }
}

pub struct Visualizer {
  glfw_event_handlers: EventHandlers,
  program: GLuint,
  vertex_buffer_obj: GLuint,
  unit_buffer_obj: GLuint,
  unit_mesh: ~[Vec3<GLfloat>],
  mat_id: GLint,
  win: Option<glfw::Window>,
  vertex_data: ~[Vec3<GLfloat>],
  mouse_pos: Vec2<f32>,
  camera: Camera,
  picker: TilePicker,
  selected_tile_pos: Option<Vec2<i32>>,
  geom: Geom,
  obj: ObjModel,
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
  win
}

impl Visualizer {
  pub fn new() -> ~Visualizer {
    let win_size = Vec2::<int>{x: 640, y: 480};
    let win = init_win(win_size);
    let geom = Geom::new();
    let mut vis = ~Visualizer {
      glfw_event_handlers: EventHandlers::new(&win),
      program: 0,
      vertex_buffer_obj: 0,
      unit_buffer_obj: 0,
      unit_mesh: ~[],
      mat_id: 0,
      win: Some(win),
      vertex_data: ~[],
      mouse_pos: Vec2::zero(),
      camera: Camera::new(),
      picker: TilePicker::new(),
      selected_tile_pos: None,
      geom: geom,
      obj: ObjModel::new("tank.obj"),
    };
    vis.init_opengl();
    vis.picker.init(&geom);
    vis.init_model();
    vis
  }


  fn win<'a>(&'a self) -> &'a glfw::Window {
    self.win.get_ref()
  }

  fn build_hex_mesh(&mut self) {
    for tile_pos in TileIterator::new() {
      let pos3d = self.geom.v2i_to_v2f(tile_pos).extend(0.0);
      for num in range(0, 6) {
        let vertex = self.geom.index_to_hex_vertex(num);
        let next_vertex = self.geom.index_to_hex_vertex(num + 1);
        let data = &mut self.vertex_data;
        data.push(pos3d + vertex.extend(0.0));
        data.push(pos3d + next_vertex.extend(0.0));
        data.push(pos3d + Vec3::zero());
      }
    }
  }

  fn init_model(&mut self) {
    self.build_hex_mesh();
    self.program = glh::compile_program(
      VERTEX_SHADER_SRC,
      FRAGMENT_SHADER_SRC,
    );
    gl::UseProgram(self.program);
    self.mat_id = glh::get_uniform(self.program, "mvp_mat");
    {
      unsafe {
        gl::GenBuffers(1, &mut self.vertex_buffer_obj);
      }
      gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_obj);
      glh::fill_current_coord_vbo(self.vertex_data);
      let pos_attr = glh::get_attr(self.program, "position");
      glh::vertex_attrib_pointer(pos_attr);
    }

    // prepare model
    {
      self.unit_mesh = self.obj.build();
      unsafe {
        gl::GenBuffers(1, &mut self.unit_buffer_obj);
      }
      gl::BindBuffer(gl::ARRAY_BUFFER, self.unit_buffer_obj);
      glh::fill_current_coord_vbo(self.unit_mesh);
    }
  }

  fn init_opengl(&mut self) {
    gl::load_with(glfw::get_proc_address);
  }

  pub fn cleanup_opengl(&self) {
    gl::DeleteProgram(self.program);
    unsafe {
      gl::DeleteBuffers(1, &self.vertex_buffer_obj);
      gl::DeleteBuffers(1, &self.unit_buffer_obj);
    }
  }

  fn draw_map(&self) {
    gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_obj);
    glh::vertex_attrib_pointer(glh::get_attr(self.program, "position"));
    glh::draw_mesh(self.vertex_data);
  }

  fn draw_model(&self) {
    gl::BindBuffer(gl::ARRAY_BUFFER, self.unit_buffer_obj);
    glh::vertex_attrib_pointer(glh::get_attr(self.program, "position"));
    glh::draw_mesh(self.unit_mesh);
  }

  pub fn draw(&self) {
    gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
    gl::UseProgram(self.program);
    glh::uniform_mat4f(self.mat_id, &self.camera.mat());
    self.draw_map();
    self.draw_model();
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
        _ => {},
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

  pub fn pick_tile(&mut self) {
    let mouse_pos = Vec2 {
      x: self.mouse_pos.x as i32,
      y: self.mouse_pos.y as i32,
    };
    let win_size = self.win().get_size();
    self.selected_tile_pos = self.picker.pick_tile(
      win_size, &self.camera, mouse_pos);
    println!("selected: {:?}", self.selected_tile_pos);
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
