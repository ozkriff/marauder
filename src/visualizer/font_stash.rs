// See LICENSE file for copyright and license details.

use crate::core::misc::add_quad_to_vec;
use crate::core::types::{MInt, Size2};
use crate::visualizer::mesh::Mesh;
use crate::visualizer::shader::Shader;
use crate::visualizer::texture::Texture;
use crate::visualizer::types::{MFloat, ScreenPos, TextureCoord, VertexCoord};
use cgmath::{Vector2, Vector3};
use stb_tt::Font;

use std::collections::HashMap;
use std::path::Path;

#[derive(Clone)]
pub struct Glyph {
    pos: ScreenPos,
    size: Size2<MInt>,
    xoff: MInt,
    yoff: MInt,
}

pub struct FontStash {
    size: MFloat,
    font: Font,
    texture: Texture,
    texture_size: MInt,
    pos: ScreenPos,
    glyphs: HashMap<char, Glyph>,
    max_h: MInt,
}

impl FontStash {
    pub fn new(font_path: &Path, size: MFloat) -> FontStash {
        let texture_size = 1024;
        let font = Font::new(font_path, size);
        let texture = Texture::new_empty(Size2 {
            w: texture_size,
            h: texture_size,
        });
        FontStash {
            size,
            font,
            texture,
            texture_size,
            pos: ScreenPos {
                v: Vector2 { x: 0, y: 0 },
            },
            glyphs: HashMap::new(),
            max_h: 0,
        }
    }

    pub fn get_glyph(&mut self, c: char) -> Glyph {
        match self.glyphs.get(&c) {
            Some(r) => r.clone(),
            None => self.add_glyph(c),
        }
    }

    pub fn get_size(&self) -> MFloat {
        self.size
    }

    pub fn get_text_size(&mut self, text: &str) -> (ScreenPos, Size2<MInt>) {
        let mut size = Size2 { w: 0, h: 0 };
        let mut pos = ScreenPos {
            v: Vector2 { x: 0, y: 0 },
        };
        for c in text.chars() {
            let glyph = self.get_glyph(c);
            let w = glyph.size.w;
            let h = glyph.size.h;
            let yoff = -glyph.yoff;
            if pos.v.y > yoff - h {
                pos.v.y = yoff - h;
            }
            if size.h < yoff {
                size.h = yoff;
            }
            size.w += w + glyph.xoff;
        }
        (pos, size)
    }

    pub fn get_mesh(&mut self, text: &str, shader: &Shader) -> Mesh {
        let mut vertex_data = Vec::new();
        let mut tex_data = Vec::new();
        let s = self.texture_size as MFloat;
        let mut i = 0.0;
        for c in text.chars() {
            let glyph = self.get_glyph(c);
            let w = glyph.size.w as MFloat;
            let h = glyph.size.h as MFloat;
            let x1 = glyph.pos.v.x as MFloat / s;
            let y1 = glyph.pos.v.y as MFloat / s;
            let x2 = x1 + w / s;
            let y2 = y1 + h / s;
            add_quad_to_vec(
                &mut tex_data,
                TextureCoord {
                    v: Vector2 { x: x1, y: y1 },
                },
                TextureCoord {
                    v: Vector2 { x: x1, y: y2 },
                },
                TextureCoord {
                    v: Vector2 { x: x2, y: y2 },
                },
                TextureCoord {
                    v: Vector2 { x: x2, y: y1 },
                },
            );
            let yoff = -glyph.yoff as MFloat;
            add_quad_to_vec(
                &mut vertex_data,
                VertexCoord {
                    v: Vector3 {
                        x: i,
                        y: yoff,
                        z: 0.0,
                    },
                },
                VertexCoord {
                    v: Vector3 {
                        x: i,
                        y: yoff - h,
                        z: 0.0,
                    },
                },
                VertexCoord {
                    v: Vector3 {
                        x: w + i,
                        y: yoff - h,
                        z: 0.0,
                    },
                },
                VertexCoord {
                    v: Vector3 {
                        x: w + i,
                        y: yoff,
                        z: 0.0,
                    },
                },
            );
            i += w + glyph.xoff as MFloat;
        }
        let mut mesh = Mesh::new(vertex_data.as_slice());
        mesh.set_texture(self.texture, tex_data.as_slice());
        mesh.prepare(shader);
        mesh
    }

    fn insert_image_to_cache(&mut self, pos: ScreenPos, size: Size2<MInt>, bitmap: Vec<u8>) {
        // let mut data = std::vec::from_elem((size.w * size.h) as u8 * 4, 0);
        let mut data = vec![0; ((size.w * size.h) * 4) as usize];
        for y in 0..size.h {
            for x in 0..size.w {
                let n = (x + y * size.w) as usize * 4;
                *data.get_mut(n + 0).unwrap() = 255;
                *data.get_mut(n + 1).unwrap() = 255;
                *data.get_mut(n + 2).unwrap() = 255;
                *data.get_mut(n + 3).unwrap() = bitmap[(x + y * size.w) as usize];
            }
        }
        self.texture.bind();
        self.texture.set_sub_image(pos.v, size, &data);
    }

    fn start_new_row(&mut self) {
        self.pos.v.y += self.max_h;
        self.pos.v.x = 0;
        self.max_h = 0;
        assert!(self.pos.v.y < self.texture_size);
    }

    fn add_glyph(&mut self, c: char) -> Glyph {
        assert!(self.glyphs.get(&c).is_none());
        let index = self.font.find_glyph_index(c);
        let (bitmap, w, h, xoff, yoff) = self.font.get_glyph(index);
        if self.pos.v.x + w > self.texture_size {
            self.start_new_row();
        }
        self.pos.v.y = std::cmp::max(h, self.pos.v.y);
        let pos = self.pos.clone();
        let size = Size2 { w, h };
        if w * h != 0 {
            self.insert_image_to_cache(pos.clone(), size, bitmap);
        }
        let xoff = if c == ' ' {
            let space_width = (self.size / 3.0) as MInt; // TODO: get from ttf
            xoff + space_width
        } else {
            xoff
        };
        self.pos.v.x += w;
        let glyph = Glyph {
            pos: pos.clone(),
            size: size.clone(),
            xoff,
            yoff,
        };
        if self.max_h < h - yoff {
            self.max_h = h - yoff;
        }
        let _ = self.glyphs.insert(c, glyph.clone());
        glyph
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
