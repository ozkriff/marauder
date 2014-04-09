// See LICENSE file for copyright and license details.

use std;
use gl;
use gl::types::{GLint, GLuint, GLsizei};
use stb_image::image;
use visualizer::shader::Shader;
use core::types::{Size2, MInt};

pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub fn new(path: ~str) -> Texture {
        load_texture(path)
    }

    pub fn new_empty(size: Size2<MInt>) -> Texture {
        get_empty_texture(size)
    }

    pub fn enable(&self, shader: &Shader) {
        let basic_texture_loc = shader.get_uniform("basic_texture") as GLint;
        gl::Uniform1ui(basic_texture_loc, 0);
        self.bind();
    }

    pub fn bind(&self) {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, self.id);
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

fn get_empty_texture(size: Size2<MInt>) -> Texture {
    let s = size.w;
    assert_eq!(size.w, size.h)
    let data = Vec::from_elem((s * s) as uint * 4, 0 as u8);
    let mut id = 0;
    unsafe {
        gl::GenTextures(1, &mut id)
    };
    gl::ActiveTexture(gl::TEXTURE0);
    gl::BindTexture(gl::TEXTURE_2D, id);
    let format = gl::RGBA;
    unsafe {
        let level = 0;
        let border = 0;
        gl::TexImage2D(
            gl::TEXTURE_2D,
            level,
            format as GLint,
            s,
            s,
            border,
            format,
            gl::UNSIGNED_BYTE,
            std::cast::transmute(data.get(0)),
        );
    }
    gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
    gl::TexParameteri(gl::TEXTURE_2D,
        gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
    Texture{id: id}
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

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
