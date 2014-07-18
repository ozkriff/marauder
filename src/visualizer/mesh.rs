// See LICENSE file for copyright and license details.

use core::types::MInt;
use visualizer::mgl::{Vbo, Vao, Triangles};
use visualizer::types::{Color3, VertexCoord, TextureCoord};
use visualizer::shader::Shader;
use visualizer::texture::Texture;

pub struct MeshId{pub id: MInt}

pub struct Mesh {
    vertex_coords_vbo: Vbo,
    color_vbo: Option<Vbo>,
    texture_coords_vbo: Option<Vbo>,
    texture: Option<Texture>,
    length: MInt,
    vao: Vao,
}

impl Mesh {
    pub fn new(data: &[VertexCoord]) -> Mesh {
        let length = data.len() as MInt;
        Mesh {
            vertex_coords_vbo: Vbo::from_data(data),
            color_vbo: None,
            texture_coords_vbo: None,
            texture: None,
            length: length,
            vao: Vao::new(),
        }
    }

    pub fn set_color(&mut self, data: &[Color3]) {
        assert_eq!(self.length, data.len() as MInt);
        self.color_vbo = Some(Vbo::from_data(data));
    }

    pub fn set_texture(&mut self, texture: Texture, data: &[TextureCoord]) {
        assert_eq!(self.length, data.len() as MInt);
        self.texture_coords_vbo = Some(Vbo::from_data(data));
        self.texture = Some(texture);
    }

    pub fn prepare(&mut self, shader: &Shader) {
        self.vao.bind();
        shader.activate();
        match self.texture_coords_vbo {
            Some(ref vbo) => {
                vbo.bind();
                shader.enable_attr("in_texture_coordinates", 2);
            },
            None => {},
        }
        match self.color_vbo {
            Some(ref vbo) => {
                vbo.bind();
                shader.enable_attr("color", 3);
            },
            None => {},
        }
        self.vertex_coords_vbo.bind();
        shader.enable_attr("in_vertex_coordinates", 3);
        self.vao.unbind();
    }

    pub fn draw(&self, shader: &Shader) {
        self.vao.bind();
        match self.texture {
            Some(ref texture) => texture.enable(shader),
            None => {},
        }
        self.vao.draw_array(Triangles, self.length);
        self.vao.unbind();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
