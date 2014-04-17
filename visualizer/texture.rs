// See LICENSE file for copyright and license details.

use std;
use gl;
use gl::types::{GLint, GLuint, GLsizei};
use stb_image::image;
use cgmath::vector::{Vector2};
use visualizer::shader::Shader;
use core::types::{Size2, MInt};

pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub fn new(path: &Path) -> Texture {
        load_texture(path)
    }

    pub fn new_empty(size: Size2<MInt>) -> Texture {
        get_empty_texture(size)
    }

    pub fn enable(&self, shader: &Shader) {
        let basic_texture_loc = shader.get_uniform("basic_texture") as GLint;
        verify!(gl::Uniform1i(basic_texture_loc, 0));
        self.bind();
    }

    pub fn bind(&self) {
        verify!(gl::ActiveTexture(gl::TEXTURE0));
        verify!(gl::BindTexture(gl::TEXTURE_2D, self.id));
    }

    pub fn set_sub_image(
        &self,
        pos: Vector2<MInt>,
        size: Size2<MInt>,
        data: &Vec<u8>
    ) {
        let bytes_per_pixel = 4;
        let expected_data_length = size.w * size.h * bytes_per_pixel;
        assert_eq!(data.len(), expected_data_length as uint);
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
                std::cast::transmute(data.get(0)),
            ));
        }
    }
}

fn load_image(path: &Path) -> image::Image<u8> {
    let str_path = match path.as_str() {
        Some(s) => s,
        None => fail!("Bad image path: {}", path.display()),
    };
    let load_result = image::load(str_path.to_owned());
    match load_result {
        image::ImageU8(image) => image,
        image::Error(message) => fail!("{}", message),
        _ => fail!("Unknown image format"),
    }
}

fn get_empty_texture(size: Size2<MInt>) -> Texture {
    let s = size.w;
    assert_eq!(size.w, size.h)
    let data = Vec::from_elem((s * s) as uint * 4, 0 as u8);
    let mut id = 0;
    unsafe {
        verify!(gl::GenTextures(1, &mut id))
    };
    verify!(gl::ActiveTexture(gl::TEXTURE0));
    verify!(gl::BindTexture(gl::TEXTURE_2D, id));
    let format = gl::RGBA;
    unsafe {
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
            std::cast::transmute(data.get(0)),
        ));
    }
    verify!(gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint));
    verify!(gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint));
    Texture{id: id}
}

fn load_texture(path: &Path) -> Texture {
    let image = load_image(path);
    let mut id = 0;
    unsafe {
        verify!(gl::GenTextures(1, &mut id))
    };
    verify!(gl::ActiveTexture(gl::TEXTURE0));
    verify!(gl::BindTexture(gl::TEXTURE_2D, id));
    let format = match image.depth {
        4 => gl::RGBA,
        3 => gl::RGB,
        _ => fail!("wrong depth"),
    };
    unsafe {
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
            std::cast::transmute(&image.data[0]),
        ));
    }
    verify!(gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint));
    verify!(gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint));
    verify!(gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint));
    verify!(gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint));
    Texture{id: id}
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
