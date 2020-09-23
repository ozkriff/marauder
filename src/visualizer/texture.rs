// See LICENSE file for copyright and license details.

use crate::core::types::{MInt, Size2};
use crate::visualizer::shader::Shader;
use cgmath::Vector2;
use gl::types::{GLint, GLsizei, GLuint};
use stb_image::image;
use std::path::Path;

#[derive(Copy, Clone)]
pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub fn new(path: &Path) -> Texture {
        // set_error_context!("loading texture", path.as_str().unwrap());
        load_texture(path)
    }

    pub fn new_empty(size: Size2<MInt>) -> Texture {
        get_empty_texture(size)
    }

    pub fn enable(&self, shader: &Shader) {
        let basic_texture_loc = shader.get_uniform("basic_texture") as GLint;
        unsafe {
            verify!(gl::Uniform1i(basic_texture_loc, 0));
        }
        self.bind();
    }

    pub fn bind(&self) {
        unsafe {
            verify!(gl::ActiveTexture(gl::TEXTURE0));
            verify!(gl::BindTexture(gl::TEXTURE_2D, self.id));
        }
    }

    pub fn set_sub_image(&self, pos: Vector2<MInt>, size: Size2<MInt>, data: &[u8]) {
        let bytes_per_pixel = 4;
        let expected_data_length = size.w * size.h * bytes_per_pixel;
        assert_eq!(data.len(), expected_data_length as usize);
        let format = gl::RGBA;
        let level = 0;
        unsafe {
            verify!(gl::TexSubImage2D(
                gl::TEXTURE_2D,
                level,
                pos.x,
                pos.y,
                size.w,
                size.h,
                format,
                gl::UNSIGNED_BYTE,
                std::mem::transmute(&data[0]),
            ));
        }
    }
}

fn load_image(path: &Path) -> image::Image<u8> {
    match image::load(path) {
        image::LoadResult::ImageU8(image) => image,
        image::LoadResult::Error(message) => panic!("{}", message),
        _ => panic!("Unknown image format"),
    }
}

fn get_empty_texture(size: Size2<MInt>) -> Texture {
    assert_eq!(size.w, size.h);
    let s = size.w;
    let mut id = 0;
    let data = vec![0; ((s * s) * 4) as usize];
    // let data: Vec<u8> = Vec::with_capacity((s * s) as usize * 4);
    unsafe {
        verify!(gl::GenTextures(1, &mut id));
        verify!(gl::ActiveTexture(gl::TEXTURE0));
        verify!(gl::BindTexture(gl::TEXTURE_2D, id));
        let format = gl::RGBA;
        let level = 0;
        let border = 0;
        verify!(gl::TexImage2D(
            gl::TEXTURE_2D,
            level,
            format as GLint,
            s,
            s,
            border,
            format,
            gl::UNSIGNED_BYTE,
            std::mem::transmute(&data[0]),
        ));
        verify!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as GLint
        ));
        verify!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as GLint
        ));
    }
    Texture { id }
}

fn load_texture(path: &Path) -> Texture {
    let image = load_image(path);
    let mut id = 0;
    unsafe {
        verify!(gl::GenTextures(1, &mut id));
        verify!(gl::ActiveTexture(gl::TEXTURE0));
        verify!(gl::BindTexture(gl::TEXTURE_2D, id));
        let format = match image.depth {
            4 => gl::RGBA,
            3 => gl::RGB,
            n => panic!("wrong depth: {}", n),
        };
        let level = 0;
        let border = 0;
        verify!(gl::TexImage2D(
            gl::TEXTURE_2D,
            level,
            format as GLint,
            image.width as GLsizei,
            image.height as GLsizei,
            border,
            format,
            gl::UNSIGNED_BYTE,
            std::mem::transmute(&image.data[0]),
        ));
        verify!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as GLint
        ));
        verify!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as GLint
        ));
        verify!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR as GLint
        ));
        verify!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR as GLint
        ));
    }
    Texture { id }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
