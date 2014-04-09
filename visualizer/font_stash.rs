// See LICENSE file for copyright and license details.

use std::cmp;
use std;
use collections::hashmap::HashMap;
use gl;
use stb_tt;
use cgmath::vector::{Vec3, Vec2};
use core::types::{Size2, Point2, MInt};
use core::misc::add_quad_to_vec;
use visualizer::texture::Texture;
use visualizer::types::MFloat;
use visualizer::mesh::Mesh;
use visualizer::shader::Shader;

struct Glyph {
    pos: Vec2<MInt>,
    size: Size2<MInt>,
    index: MInt,
    xoff: MInt,
    yoff: MInt,
}

pub struct FontStash {
    size: MFloat,
    font: stb_tt::Font,
    texture: Texture,
    texture_size: MInt,
    pos: Point2<MInt>,
    glyphs: HashMap<char, Glyph>,
    mesh: Mesh,
}

impl FontStash {
    pub fn new(font_path: &str, size: MFloat) -> FontStash {
        // TODO: use updateble mesh
        let texture_size = 1024;
        let font = stb_tt::Font::new(font_path, size);
        let texture = Texture::new_empty(
            Size2{w: texture_size, h: texture_size});
        let vertex_data = Vec::new();
        let mesh = Mesh::new(vertex_data.as_slice());
        FontStash {
            size: size,
            font: font,
            texture: texture,
            texture_size: texture_size,
            pos: Vec2{x: 0, y: 0},
            glyphs: HashMap::new(),
            mesh: mesh,
        }
    }

    pub fn get_glyph(&mut self, c: char) -> Glyph {
        match self.glyphs.find(&c) {
            Some(&r) => r,
            None => self.add_glyph(c),
        }
    }

    pub fn get_mesh(&mut self, text: &str, shader: &Shader) -> Mesh {
        // TODO: create mesh in c-tor, update mesh vertices data here
        let mut vertex_data = Vec::new();
        let mut tex_data = Vec::new();
        let s = self.texture_size as MFloat;
        let mut i = 0.0;
        for c in text.chars() {
            let glyph = self.get_glyph(c);
            let x1 = glyph.pos.x as MFloat / s;
            let y1 = glyph.pos.y as MFloat / s;
            let w = glyph.size.w as MFloat / s;
            let h = glyph.size.h as MFloat / s;
            let x2 = x1 + w;
            let y2 = y1 + h;
            add_quad_to_vec(
                &mut tex_data,
                Vec2{x: x1, y: y1},
                Vec2{x: x1, y: y2},
                Vec2{x: x2, y: y2},
                Vec2{x: x2, y: y1},
            );
            let yoff = glyph.yoff as MFloat / s;
            add_quad_to_vec(
                &mut vertex_data,
                Vec3{x: i, y: yoff, z: -0.01},
                Vec3{x: i, y: yoff + h, z: -0.01},
                Vec3{x: w + i, y: yoff + h, z: -0.01},
                Vec3{x: w + i, y: yoff, z: -0.01},
            );
            i += w;
        }
        let mut mesh = Mesh::new(vertex_data.as_slice());
        mesh.set_texture(self.texture, tex_data.as_slice());
        mesh.prepare(shader);
        mesh
    }

    fn add_glyph(&mut self, c: char) -> Glyph {
        assert!(self.glyphs.find(&c).is_none());
        let index = self.font.find_glyph_index(c);
        let (bitmap, w, h, xoff, yoff) = self.font.get_glyph(index);
        assert!(w != 0 && h != 0);
        assert!(self.pos.x + w < self.texture_size); // TODO: rows
        let mut data = Vec::from_elem((w * h) as uint * 4, 0 as u8);
        for y in range(0, h) {
            for x in range(0, w) {
                let n = (x + y * w) as uint * 4;
                *data.get_mut(n + 0) = 0;
                *data.get_mut(n + 1) = 0;
                *data.get_mut(n + 2) = 0;
                *data.get_mut(n + 3) = bitmap[(x + y * w) as uint];
            }
        }
        self.texture.bind();
        let format = gl::RGBA;
        self.pos.y = cmp::max(h, self.pos.y);
        let pos = self.pos;
        let size = Size2{w: w, h: h};
        unsafe {
            let level = 0;
            // TODO: use some texure::Texture method
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                level,
                pos.x,
                pos.y,
                size.w,
                size.h,
                format,
                gl::UNSIGNED_BYTE,
                std::cast::transmute(data.get(0)),
            );
        }
        self.pos.x += w;
        let glyph = Glyph {
            pos: pos,
            size: size,
            index: index,
            xoff: xoff,
            yoff: yoff,
        };
        self.glyphs.insert(c, glyph);
        glyph
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
