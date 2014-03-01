// See LICENSE file for copyright and license details.

use std;
use gl;
use gl::types::{
    GLint,
    GLchar,
    GLuint,
    GLenum,
};
use core::core_types::{
    MInt,
};
use core::misc::read_file;

fn c_str(s: &str) -> *GLchar {
    unsafe {
        s.to_c_str().unwrap()
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

    pub fn enable_attr(&self, name:&str, components_count: MInt) {
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

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
