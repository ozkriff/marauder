// See LICENSE file for copyright and license details.

use gl_helpers::{
    Texture,
    Shader,
    Vbo,
    Vao,
    draw_mesh,
    Triangles,
};
use gl_types::{
    Color3,
    VertexCoord,
    TextureCoord,
};
use core_types::Int;

pub struct Mesh {
    priv vertex_coords_vbo: Vbo,
    priv color_vbo: Option<Vbo>,
    priv texture_coords_vbo: Option<Vbo>,
    priv texture: Option<Texture>,
    priv length: Int,
    priv vao: Vao,
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            vertex_coords_vbo: Vbo(0),
            color_vbo: None,
            texture_coords_vbo: None,
            texture: None,
            length: 0,
            vao: Vao(0),
        }
    }

    pub fn set_vertex_coords(&mut self, data: &[VertexCoord]) {
        self.length = data.len() as Int;
        self.vertex_coords_vbo = Vbo::from_data(data);
    }

    pub fn set_color(&mut self, data: &[Color3]) {
        assert_eq!(self.length, data.len() as Int);
        self.color_vbo = Some(Vbo::from_data(data));
    }

    pub fn set_texture(&mut self, texture: Texture, data: &[TextureCoord]) {
        assert_eq!(self.length, data.len() as Int);
        self.texture_coords_vbo = Some(Vbo::from_data(data));
        self.texture = Some(texture);
    }

    pub fn prepare(&mut self, shader: &Shader) {
        self.vao = Vao::new();
        self.vao.bind();
        shader.activate();
        if !self.texture_coords_vbo.is_none() {
            self.texture_coords_vbo.get_ref().bind();
            let p = shader.get_attr("in_texture_coordinates");
            p.enable();
            p.vertex_pointer(2);
        }
        if !self.color_vbo.is_none() {
            self.color_vbo.get_ref().bind();
            let p = shader.get_attr("color");
            p.enable();
            p.vertex_pointer(3);
        }
        self.vertex_coords_vbo.bind();
        let p = shader.get_attr("in_vertex_coordinates");
        p.enable();
        p.vertex_pointer(3);
        self.vao.unbind();
    }

    pub fn draw(&self, shader: &Shader) {
        self.vao.bind();
        shader.activate();
        if !self.texture.is_none() {
            self.texture.unwrap().enable(shader);
        }
        draw_mesh(Triangles, self.length);
        self.vao.unbind();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
