// See LICENSE file for copyright and license details.

use std;
use gl;
use gl::types::{GLint, GLchar, GLuint, GLenum};
use cgmath::matrix::{Matrix, Matrix4};
use core::types::MInt;
use core::misc::read_file;
use visualizer::types::{MatId, MFloat, ColorId, Color4};

pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn new(vs_path: &Path, fs_path: &Path) -> Shader {
        compile_program(
            read_file(vs_path),
            read_file(fs_path),
        )
    }

    pub fn activate(&self) {
        verify!(gl::UseProgram(self.id));
    }

    pub fn enable_attr(&self, name: &str, components_count: MInt) {
        let mut attr_id = 0;
        name.with_c_str(|name| {
            unsafe {
                attr_id = verify!(gl::GetAttribLocation(self.id, name));
            }
        });
        verify!(gl::EnableVertexAttribArray(attr_id as GLuint));
        let normalized = gl::FALSE;
        let stride = 0;
        unsafe {
            verify!(gl::VertexAttribPointer(
                attr_id as GLuint,
                components_count,
                gl::FLOAT,
                normalized,
                stride,
                std::ptr::null(),
            ));
        }
    }

    pub fn uniform_mat4f(&self, mat_id: MatId, mat: &Matrix4<MFloat>) {
        unsafe {
            verify!(gl::UniformMatrix4fv(
                mat_id.id as MInt, 1, gl::FALSE, mat.cr(0, 0)));
        }
    }

    pub fn uniform_color(&self, color_id: ColorId, color: Color4) {
        unsafe {
            let data_ptr = std::cast::transmute(&color);
            verify!(gl::Uniform4fv(color_id.id as MInt, 1, data_ptr));
        }
    }

    pub fn get_uniform(&self, name: &str) -> GLuint {
        name.with_c_str(|name| {
            unsafe {
                verify!(gl::GetUniformLocation(self.id, name) as GLuint)
            }
        })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        verify!(gl::DeleteProgram(self.id));
    }
}

fn compile_shader(src: &str, shader_type: GLenum) -> GLuint {
    let shader = verify!(gl::CreateShader(shader_type));
    unsafe {
        src.with_c_str(|src| {
            verify!(gl::ShaderSource(shader, 1, &src, std::ptr::null()));
        });
        verify!(gl::CompileShader(shader));
        let mut status = gl::FALSE as GLint;
        verify!(gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status));
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            verify!(gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len));
            // subtract 1 to skip the trailing null character
            let mut buf = Vec::from_elem(len as uint - 1, 0u8);
            verify!(gl::GetShaderInfoLog(shader, len, std::ptr::mut_null(),
                buf.as_mut_ptr() as *mut GLchar));
            fail!("compile_shader(): "
                + std::str::raw::from_utf8(buf.as_slice()));
        }
    }
    shader
}

fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    let program = verify!(gl::CreateProgram());
    verify!(gl::AttachShader(program, vertex_shader));
    verify!(gl::AttachShader(program, fragment_shader));
    verify!(gl::LinkProgram(program));
    unsafe {
        let mut status = gl::FALSE as GLint;
        verify!(gl::GetProgramiv(program, gl::LINK_STATUS, &mut status));
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            verify!(gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len));
            // subtract 1 to skip the trailing null character
            let mut buf = Vec::from_elem(len as uint - 1, 0u8);
            verify!(gl::GetProgramInfoLog(program, len, std::ptr::mut_null(),
                buf.as_mut_ptr() as *mut GLchar));
            fail!("link_program(): "
                + std::str::raw::from_utf8(buf.as_slice()));
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
    verify!(gl::DeleteShader(fragment_shader));
    verify!(gl::DeleteShader(vertex_shader));
    Shader{id: program}
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
