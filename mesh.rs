// See LICENSE file for copyright and license details.

use gl_helpers::{
    Texture,
    Shader,
    Vbo,
    gen_buffer,
    bind_buffer,
    fill_current_coord_vbo,
    fill_current_color_vbo,
    fill_current_texture_coords_vbo,
    enable_texture,
    get_attr,
    vertex_attrib_pointer,
    delete_buffer,
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
        self.vertex_coords_vbo = gen_buffer();
        bind_buffer(&self.vertex_coords_vbo);
        fill_current_coord_vbo(data);
    }

    pub fn set_color(&mut self, data: &[Color3]) {
        assert_eq!(self.length, data.len() as Int);
        self.color_vbo = Some(gen_buffer());
        bind_buffer(&self.color_vbo.unwrap());
        fill_current_color_vbo(data);
    }

    pub fn set_texture_coords(&mut self, data: &[TextureCoord]) {
        assert_eq!(self.length, data.len() as Int);
        self.texture_coords_vbo = Some(gen_buffer());
        bind_buffer(&self.texture_coords_vbo.unwrap());
        fill_current_texture_coords_vbo(data);
    }

    pub fn set_texture(&mut self, texture: Texture) {
        self.texture = Some(texture);
    }

    pub fn draw(&self, shader: &Shader) {
        if !self.texture.is_none() {
            enable_texture(shader, &self.texture.unwrap());
        }
        if !self.texture_coords_vbo.is_none() {
            bind_buffer(&self.texture_coords_vbo.unwrap());
            let p = get_attr(shader, "in_texture_coordinates");
            vertex_attrib_pointer(p, 2);
        }
        if !self.color_vbo.is_none() {
            bind_buffer(&self.color_vbo.unwrap());
            let p = get_attr(shader, "color");
            vertex_attrib_pointer(p, 3);
        }
        bind_buffer(&self.vertex_coords_vbo);
        let p = get_attr(shader, "in_vertex_coordinates");
        vertex_attrib_pointer(p, 3);
        draw_mesh(self.length);
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        delete_buffer(&self.vertex_coords_vbo);
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
