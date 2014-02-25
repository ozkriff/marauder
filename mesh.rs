// See LICENSE file for copyright and license details.

use gl_helpers::{
    Texture,
    Shader,
    Vbo,
    fill_current_coord_vbo,
    fill_current_color_vbo,
    fill_current_texture_coords_vbo,
    get_attr,
    draw_mesh,
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
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            vertex_coords_vbo: Vbo(0),
            color_vbo: None,
            texture_coords_vbo: None,
            texture: None,
            length: 0,
        }
    }

    pub fn set_vertex_coords(&mut self, data: &[VertexCoord]) {
        self.length = data.len() as Int;
        self.vertex_coords_vbo = Vbo::new();
        self.vertex_coords_vbo.bind();
        fill_current_coord_vbo(data);
    }

    pub fn set_color(&mut self, data: &[Color3]) {
        assert_eq!(self.length, data.len() as Int);
        self.color_vbo = Some(Vbo::new());
        self.color_vbo.get_ref().bind();
        fill_current_color_vbo(data);
    }

    pub fn set_texture_coords(&mut self, data: &[TextureCoord]) {
        assert_eq!(self.length, data.len() as Int);
        self.texture_coords_vbo = Some(Vbo::new());
        self.texture_coords_vbo.get_ref().bind();
        fill_current_texture_coords_vbo(data);
    }

    pub fn set_texture(&mut self, texture: Texture) {
        self.texture = Some(texture);
    }

    pub fn draw(&self, shader: &Shader) {
        if !self.texture.is_none() {
            self.texture.unwrap().enable(shader);
        }
        if !self.texture_coords_vbo.is_none() {
            self.texture_coords_vbo.get_ref().bind();
            let p = get_attr(shader, "in_texture_coordinates");
            p.vertex_pointer(2);
        }
        if !self.color_vbo.is_none() {
            self.color_vbo.get_ref().bind();
            let p = get_attr(shader, "color");
            p.vertex_pointer(3);
        }
        self.vertex_coords_vbo.bind();
        let p = get_attr(shader, "in_vertex_coordinates");
        p.vertex_pointer(3);
        draw_mesh(self.length);
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
