// See LICENSE file for copyright and license details.

use crate::core::types::MInt;
use crate::visualizer::types::{Normal, TextureCoord, VertexCoord};
use cgmath::{Vector2, Vector3};

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::{FromStr, Split, SplitWhitespace};

struct Face {
    vertex: [MInt; 3],
    texture: [MInt; 3],
    normal: [MInt; 3],
}

pub struct Model {
    coords: Vec<VertexCoord>,
    normals: Vec<Normal>,
    texture_coords: Vec<TextureCoord>,
    faces: Vec<Face>,
}

fn parse_word<T: FromStr>(words: &mut SplitWhitespace) -> T
where
    T::Err: std::fmt::Debug,
{
    let str = words.next().expect("Can not read next word");
    str.parse().expect("Can not parse word")
}

fn parse_charsplit<T: FromStr>(words: &mut Split<char>) -> T
where
    T::Err: std::fmt::Debug,
{
    let str = words.next().expect("Can not read next word");
    str.parse().expect("Can not parse word")
}

impl Model {
    pub fn new(path: &Path) -> Model {
        // set_error_context!("loading obj", path.as_str().unwrap());
        let mut obj = Model {
            coords: Vec::new(),
            normals: Vec::new(),
            texture_coords: Vec::new(),
            faces: Vec::new(),
        };
        obj.read(path);
        obj
    }

    fn read_v(words: &mut SplitWhitespace) -> VertexCoord {
        VertexCoord {
            v: Vector3 {
                x: parse_word(words),
                y: parse_word(words),
                z: parse_word(words),
            },
        }
    }

    fn read_vn(words: &mut SplitWhitespace) -> Normal {
        Normal {
            v: Vector3 {
                x: parse_word(words),
                y: parse_word(words),
                z: parse_word(words),
            },
        }
    }

    fn read_vt(words: &mut SplitWhitespace) -> TextureCoord {
        TextureCoord {
            v: Vector2 {
                x: parse_word(words),
                y: 1.0 - parse_word::<f32>(words), // flip
            },
        }
    }

    fn read_f(words: &mut SplitWhitespace) -> Face {
        let mut face = Face {
            vertex: [0, 0, 0],
            texture: [0, 0, 0],
            normal: [0, 0, 0],
        };
        let mut i = 0;
        for group in words {
            let mut w = group.split('/');
            face.vertex[i] = parse_charsplit(&mut w);
            face.texture[i] = parse_charsplit(&mut w);
            face.normal[i] = parse_charsplit(&mut w);
            i += 1;
        }
        face
    }

    fn read_line(&mut self, line: &str) {
        let mut words = line.split_whitespace();
        fn is_correct_tag(tag: &str) -> bool {
            !tag.is_empty() && !tag.starts_with('#')
        }
        match words.next() {
            Some(tag) if is_correct_tag(tag) => {
                let w = &mut words;
                match tag {
                    "v" => self.coords.push(Model::read_v(w)),
                    "vn" => self.normals.push(Model::read_vn(w)),
                    "vt" => self.texture_coords.push(Model::read_vt(w)),
                    "f" => self.faces.push(Model::read_f(w)),
                    _ => {}
                }
            }
            _ => {}
        };
    }

    fn read(&mut self, path: &Path) {
        let file = BufReader::new(File::open(path).unwrap());
        for line in file.lines() {
            match line {
                Ok(line) => self.read_line(&line),
                Err(msg) => panic!("Obj: read error: {}", msg),
            }
        }
    }

    pub fn build(&self) -> Vec<VertexCoord> {
        let mut mesh = Vec::new();
        for face in self.faces.iter() {
            for i in 0..3 {
                let vertex_id = face.vertex[i] - 1;
                mesh.push(self.coords[vertex_id as usize].clone());
            }
        }
        mesh
    }

    pub fn build_tex_coord(&self) -> Vec<TextureCoord> {
        let mut tex_coords = Vec::new();
        for face in self.faces.iter() {
            for i in 0..3 {
                let texture_coord_id = face.texture[i] as usize - 1;
                tex_coords.push(self.texture_coords[texture_coord_id].clone());
            }
        }
        tex_coords
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
