// See LICENSE file for copyright and license details.

use gl_helpers::{
    Vbo,
    Vao,
    Triangles,
};
use gl_types::{
    Color3,
    VertexCoord,
    TextureCoord,
};
use core_types::MInt;
use shader::Shader;
use texture::Texture;

pub struct Mesh {
    priv vertex_coords_vbo: Vbo,
    priv color_vbo: Option<Vbo>,
    priv texture_coords_vbo: Option<Vbo>,
    priv texture: Option<Texture>,
    priv length: MInt,
    priv vao: Vao,
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
        if !self.texture_coords_vbo.is_none() {
            self.texture_coords_vbo.get_ref().bind();
            shader.enable_attr("in_texture_coordinates", 2);
        }
        if !self.color_vbo.is_none() {
            self.color_vbo.get_ref().bind();
            shader.enable_attr("color", 3);
        }
        self.vertex_coords_vbo.bind();
        shader.enable_attr("in_vertex_coordinates", 3);
        self.vao.unbind();
    }

    pub fn draw(&self, shader: &Shader) {
        self.vao.bind();
        shader.activate();
        if !self.texture.is_none() {
            self.texture.unwrap().enable(shader);
        }
        self.vao.draw_array(Triangles, self.length);
        self.vao.unbind();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
