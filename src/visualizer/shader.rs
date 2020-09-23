// See LICENSE file for copyright and license details.

use crate::cgmath::Matrix;
use crate::core::misc::read_file;
use crate::core::types::MInt;
use crate::visualizer::types::{Color4, ColorId, MFloat, MatId};
use cgmath::Matrix4;
use gl::types::{GLchar, GLenum, GLint, GLuint};
use glfw::with_c_str;
use std::path::Path;

pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn new(vs_path: &Path, fs_path: &Path) -> Shader {
        // set_error_context!("loading shader (vs)", vs_path.as_str().unwrap());
        // set_error_context!("loading shader (fs)", fs_path.as_str().unwrap());
        compile_program(&read_file(vs_path), &read_file(fs_path))
    }

    pub fn activate(&self) {
        unsafe {
            verify!(gl::UseProgram(self.id));
        }
    }

    pub fn enable_attr(&self, name: &str, components_count: MInt) {
        let mut attr_id = 0;
        // let c_name = CString::new(name).unwrap();
        with_c_str(name, |n| unsafe {
            attr_id = verify!(gl::GetAttribLocation(self.id, n));
        });
        unsafe {
            verify!(gl::EnableVertexAttribArray(attr_id as GLuint));
        }
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
                mat_id.id as MInt,
                1,
                gl::FALSE,
                mat.as_ptr(),
            ));
        }
    }

    pub fn uniform_color(&self, color_id: ColorId, color: Color4) {
        unsafe {
            let data_ptr = std::mem::transmute(&color);
            verify!(gl::Uniform4fv(color_id.id as MInt, 1, data_ptr));
        }
    }

    pub fn get_uniform(&self, name: &str) -> GLuint {
        with_c_str(name, |name| unsafe {
            verify!(gl::GetUniformLocation(self.id, name) as GLuint)
        })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            verify!(gl::DeleteProgram(self.id));
        }
    }
}

fn compile_shader(src: &str, shader_type: GLenum) -> GLuint {
    unsafe {
        let shader = verify!(gl::CreateShader(shader_type));
        with_c_str(src, |s| {
            verify!(gl::ShaderSource(shader, 1, &s, std::ptr::null()));
        });
        verify!(gl::CompileShader(shader));
        let mut status = gl::FALSE as GLint;
        verify!(gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status));
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            verify!(gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len));
            // subtract 1 to skip the trailing null character
            // let mut buf = std::vec::from_elem(len as u8 - 1, 0);
            let mut buf = vec![0_u8; (len) as usize];
            verify!(gl::GetShaderInfoLog(
                shader,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar
            ));
            panic!("compile_shader: {}", String::from_utf8(buf).unwrap());
        }
        shader
    }
}

fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    unsafe {
        let program = verify!(gl::CreateProgram());
        verify!(gl::AttachShader(program, vertex_shader));
        verify!(gl::AttachShader(program, fragment_shader));
        verify!(gl::LinkProgram(program));
        let mut status = gl::FALSE as GLint;
        verify!(gl::GetProgramiv(program, gl::LINK_STATUS, &mut status));
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            verify!(gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len));
            // subtract 1 to skip the trailing null character
            let mut buf = std::vec::from_elem(len as u8 - 1, 0_usize);
            verify!(gl::GetProgramInfoLog(
                program,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar
            ));
            panic!("link_program: {}", String::from_utf8(buf).unwrap());
        }
        program
    }
}

fn compile_program(vertex_shader_src: &str, frag_shader_src: &str) -> Shader {
    let vertex_shader = compile_shader(vertex_shader_src, gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(frag_shader_src, gl::FRAGMENT_SHADER);
    let program = link_program(vertex_shader, fragment_shader);
    // mark shaders for deletion after program deletion
    unsafe {
        verify!(gl::DeleteShader(fragment_shader));
        verify!(gl::DeleteShader(vertex_shader));
    }
    Shader { id: program }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
