// See LICENSE file for copyright and license details.

// Marauder is grid-less turn-based strategy game.

extern mod glfw;
extern mod gl;
extern mod cgmath;

use gl::types::{
  GLenum,
  GLuint,
  GLint,
  GLchar,
  GLfloat,
  GLsizeiptr,
  GLboolean
};
use cgmath::matrix::{
  Matrix,
  Mat4,
  Mat3,
  ToMat4
};
use cgmath::vector::{
  Vec3,
  Vec2
};
use cgmath::projection;
use cgmath::angle;

#[link(name = "glfw")]
#[link(name = "m")]
extern {}

/// usage: let s = mvp_matrix; println(type_of(&s));
fn type_of<T>(_: &T) -> &'static str {
  unsafe {
    (*std::unstable::intrinsics::get_tydesc::<T>()).name
  }
}

fn print_mat4(name: &str, mat: Mat4<f32>) {
  println!("{}:", name);
  for i in range(0u, 4) {
    for j in range(0u, 4) {
      print!("{} ", *mat.cr(i, j));
    }
    println("");
  }
  println("");
}

static mut MOUSE_POS: Vec2<f32> = Vec2{x: 0.0f32, y: 0.0};
static mut CAMERA_POS: Vec3<f32> = Vec3{x: 0.0f32, y: 0.0, z: 0.0};

static WIN_SIZE: Vec2<uint> = Vec2{x: 640, y: 480};
static VERTICES_COUNT: i32 = 3 * 2;

static VERTEX_DATA: [GLfloat, ..VERTICES_COUNT * 3] = [
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

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
  let shader = gl::CreateShader(ty);
  unsafe {
    gl::ShaderSource(shader, 1, &src.to_c_str().unwrap(), std::ptr::null());
    gl::CompileShader(shader);

    let mut status = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

    if status != (gl::TRUE as GLint) {
      let mut len = 0;
      gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
      // subtract 1 to skip the trailing null character
      let mut buf = std::vec::from_elem(len as uint - 1, 0u8);
      gl::GetShaderInfoLog(shader, len, std::ptr::mut_null(),
        std::vec::raw::to_mut_ptr(buf) as *mut GLchar
      );
      fail!("compile_shader(): " + std::str::raw::from_utf8(buf));
    }
  }
  shader
}

fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
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
        std::vec::raw::to_mut_ptr(buf) as *mut GLchar
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
  vertex_shader: GLuint,
  fragment_shader: GLuint,
  program: GLuint,
  vertex_array_obj: GLuint,
  vertex_buffer_obj: GLuint,
  matrix_id: GLint,
  window: glfw::Window
}

impl Win {
  fn new() -> Win {
    // TODO: Move to opengl_init method
    // glfw::window_hint::context_version(3, 2);

    let mut win =  Win{
      vertex_shader: 0,
      fragment_shader: 0,
      program: 0,
      vertex_array_obj: 0,
      vertex_buffer_obj: 0,
      matrix_id: 0,
      window: glfw::Window::create(
        WIN_SIZE.x,
        WIN_SIZE.y,
        "OpenGL",
        glfw::Windowed
      ).unwrap()
    };

    win.window.make_context_current();
    win.window.set_cursor_pos_callback(~CursorPosContext);
    win.window.set_key_callback(~KeyContext);

    // Load the OpenGL function pointers
    gl::load_with(glfw::get_proc_address);

    win.vertex_shader = compile_shader(
      VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
    win.fragment_shader = compile_shader(
      FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);
    win.program = link_program(win.vertex_shader, win.fragment_shader);

    unsafe {
      // Create Vertex Array Object
      gl::GenVertexArrays(1, &mut win.vertex_array_obj);
      gl::BindVertexArray(win.vertex_array_obj);

      // Create a Vertex Buffer Object and copy the vertex data to it
      gl::GenBuffers(1, &mut win.vertex_buffer_obj);
      gl::BindBuffer(gl::ARRAY_BUFFER, win.vertex_buffer_obj);
      let float_size = std::mem::size_of::<GLfloat>();
      let vertices_ptr = (VERTEX_DATA.len() * float_size) as GLsizeiptr;
      gl::BufferData(
        gl::ARRAY_BUFFER,
        vertices_ptr,
        std::cast::transmute(&VERTEX_DATA[0]),
        gl::STATIC_DRAW
      );

      gl::UseProgram(win.program);
      gl::BindFragDataLocation(
        win.program, 0, "out_color".to_c_str().unwrap());

      // Specify the layout of the vertex data
      let pos_attr = gl::GetAttribLocation(
        win.program, "position".to_c_str().unwrap()) as GLuint;
      gl::EnableVertexAttribArray(pos_attr);
      let size = 3;
      let normalized = gl::FALSE as GLboolean;
      let stride = 0;
      gl::VertexAttribPointer(
        pos_attr,
        size,
        gl::FLOAT,
        normalized,
        stride,
        std::ptr::null()
      );

      win.matrix_id = gl::GetUniformLocation(
        win.program, "model_view_proj_matrix".to_c_str().unwrap()
      );
    }
 
    win
  }

  fn cleanup(&self) {
    gl::DeleteProgram(self.program);
    gl::DeleteShader(self.fragment_shader);
    gl::DeleteShader(self.vertex_shader);
    unsafe {
      gl::DeleteBuffers(1, &self.vertex_buffer_obj);
      gl::DeleteVertexArrays(1, &self.vertex_array_obj);
    }
  }

  fn update_matrices(&self) {
    let fov = angle::deg(45.0f32);
    let ratio = 4.0 / 3.0;
    let display_range_min = 0.1;
    let display_range_max = 100.0;
    let projection_matrix = projection::perspective(
      fov, ratio, display_range_min, display_range_max
    );

    let mut mvp_matrix = projection_matrix;
    unsafe {
      mvp_matrix = tr(mvp_matrix, Vec3{x: 0.0f32, y: 0.0, z: -10.0f32});
      mvp_matrix = rot_x(mvp_matrix, MOUSE_POS.y / 100.0);
      mvp_matrix = rot_y(mvp_matrix, MOUSE_POS.x / 100.0);
      mvp_matrix = tr(mvp_matrix, CAMERA_POS);
    }

    unsafe {
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
  std::rt::start_on_main_thread(argc, argv, main)
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
