// See LICENSE file for copyright and license details.

use std;
use gl;
use gl::types::{
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
    Vec2,
    Vec3,
};
use cgmath::angle;
use stb_image::image;
use misc::{
    deg_to_rad,
    read_file,
};
use gl_types::{
    Float,
    MatId,
};
use core_types::{
    Size2,
    Int,
};

pub use load_gl_funcs_with = gl::load_with;

fn c_str(s: &str) -> *GLchar {
    unsafe {
        s.to_c_str().unwrap()
    }
}

fn compile_shader(src: &str, shader_type: GLenum) -> GLuint {
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

fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
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

fn compile_program(vertex_shader_src: &str, frag_shader_src: &str) -> Shader {
    let vertex_shader = compile_shader(
        vertex_shader_src, gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(
        frag_shader_src, gl::FRAGMENT_SHADER);
    let program = link_program(vertex_shader, fragment_shader);
    // mark shaders for deletion after program deletion
    gl::DeleteShader(fragment_shader);
    gl::DeleteShader(vertex_shader);
    Shader{id: program}
}

pub enum MeshRenderMode {
    Triangles,
    Lines,
}

impl MeshRenderMode {
    fn to_gl_type(&self) -> GLuint {
        match *self {
            Triangles => gl::TRIANGLES,
            Lines => gl::LINES,
        }
    }
}

pub fn uniform_mat4f(mat_id: MatId, mat: &Mat4<Float>) {
    unsafe {
        let MatId(id) = mat_id;
        gl::UniformMatrix4fv(id as Int, 1, gl::FALSE, mat.cr(0, 0));
    }
}

pub fn tr(m: Mat4<Float>, v: Vec3<Float>) -> Mat4<Float> {
    let mut t = Mat4::<Float>::identity();
    *t.mut_cr(3, 0) = v.x;
    *t.mut_cr(3, 1) = v.y;
    *t.mut_cr(3, 2) = v.z;
    m.mul_m(&t)
}

pub fn rot_x(m: Mat4<Float>, angle: Float) -> Mat4<Float> {
    let rad = angle::rad(deg_to_rad(angle));
    let r = Mat3::from_angle_x(rad).to_mat4();
    m.mul_m(&r)
}

pub fn rot_z(m: Mat4<Float>, angle: Float) -> Mat4<Float> {
    let rad = angle::rad(deg_to_rad(angle));
    let r = Mat3::from_angle_z(rad).to_mat4();
    m.mul_m(&r)
}

pub fn init_opengl() {
    gl::Enable(gl::DEPTH_TEST);
}

pub fn set_clear_color(r: Float, g: Float, b: Float) {
    gl::ClearColor(r, g, b, 1.0);
}

pub fn clear_screen() {
    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
}

pub fn set_viewport(size: Size2<Int>) {
    gl::Viewport(0, 0, size.w, size.h);
}

pub struct Texture {
    priv id: GLuint,
}

impl Texture {
    pub fn new(path: ~str) -> Texture {
        load_texture(path)
    }

    pub fn enable(&self, shader: &Shader) {
        let basic_texture_loc = shader.get_uniform("basic_texture") as GLint;
        gl::Uniform1ui(basic_texture_loc, 0);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, self.id);
    }
}

pub struct Shader {
    priv id: GLuint,
}

impl Shader {
    pub fn new(vs: &str, fs: &str) -> Shader {
        compile_program(
            read_file(&Path::new(vs)),
            read_file(&Path::new(fs)),
        )
    }

    pub fn activate(&self) {
        gl::UseProgram(self.id);
    }

    pub fn enable_attr(&self, name:&str, components_count: Int) {
        let mut attr_id;
        unsafe {
            attr_id = gl::GetAttribLocation(self.id, c_str(name));
        }
        gl::EnableVertexAttribArray(attr_id as GLuint);
        let normalized = gl::FALSE;
        let stride = 0;
        unsafe {
            gl::VertexAttribPointer(
                attr_id as GLuint,
                components_count,
                gl::FLOAT,
                normalized,
                stride,
                std::ptr::null(),
            );
        }
    }

    pub fn get_uniform(&self, name: &str) -> GLuint {
        unsafe {
            gl::GetUniformLocation(self.id, c_str(name)) as GLuint
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        gl::DeleteProgram(self.id);
    }
}

pub struct Vao {
    priv id: GLuint,
}

impl Vao {
    pub fn new() -> Vao {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        let vao = Vao{id: id};
        vao.bind();
        gl::EnableVertexAttribArray(id);
        vao
    }

    pub fn bind(&self) {
        gl::BindVertexArray(self.id);
    }

    pub fn unbind(&self) {
        gl::BindVertexArray(0);
    }

    pub fn draw_array(&self, mesh_mode: MeshRenderMode, faces_count: Int) {
        let starting_index = 0;
        let vertices_count = faces_count * 3;
        let mode = mesh_mode.to_gl_type();
        gl::DrawArrays(mode, starting_index, vertices_count);
    }
}

impl Drop for Vao {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

pub struct Vbo {
    priv id: GLuint,
}

fn get_new_vbo_id() -> GLuint {
    let mut id = 0;
    unsafe {
        gl::GenBuffers(1, &mut id);
    }
    id
}

impl Vbo {
    pub fn from_data<T>(data: &[T]) -> Vbo {
        let vbo = Vbo{id: get_new_vbo_id()};
        vbo.bind();
        let size = std::mem::size_of::<T>();
        let buf_size = (data.len() * size) as GLsizeiptr;
        unsafe {
            let data_ptr = std::cast::transmute(&data[0]);
            gl::BufferData(gl::ARRAY_BUFFER, buf_size, data_ptr, gl::STATIC_DRAW);
        }
        vbo
    }

    pub fn bind(&self) {
        gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
    }
}

impl Drop for Vbo {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

fn load_image(path: ~str) -> image::Image<u8> {
    let load_result = image::load(path);
    match load_result {
        image::ImageU8(image) => image,
        image::Error(message) => fail!("{}", message),
        _ => fail!("Unknown image format"),
    }
}

fn load_texture(path: ~str) -> Texture {
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
    Texture{id: id}
}

pub fn get_vec2_from_pixel(
    win_size: Size2<Int>,
    mouse_pos: Vec2<Int>,
) -> Option<Vec2<Int>> {
    let height = win_size.h;
    let reverted_h = height - mouse_pos.y;
    let data: [u8, ..4] = [0, 0, 0, 0]; // mut
    unsafe {
        let data_ptr = std::cast::transmute(&data[0]);
        gl::ReadPixels(
            mouse_pos.x, reverted_h, 1, 1,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data_ptr
        );
    }
    if data[2] != 0 {
        Some(Vec2{x: data[0] as Int, y: data[1] as Int})
    } else {
        None
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
