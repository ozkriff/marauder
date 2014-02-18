// See LICENSE file for copyright and license details.

use std::str::Words;
use std::io::{
  BufferedReader,
  File
};
use cgmath::vector::{
  Vec3,
  Vec2,
};
use core_types::{
  Bool,
  Int,
};
use gl_types::{
  VertexCoord,
  TextureCoord,
  Normal,
};

struct Face {
  vertex: [Int, ..3],
  texture: [Int, ..3],
  normal: [Int, ..3],
}

pub struct Model {
  coords: ~[VertexCoord],
  normals: ~[Normal],
  texture_coords: ~[TextureCoord],
  faces: ~[Face],
}

// TODO: unwrap() -> ...
impl Model {
  pub fn new(filename: &str) -> Model {
    let mut obj = Model {
      coords: ~[],
      normals: ~[],
      texture_coords: ~[],
      faces: ~[],
    };
    obj.read(filename);
    obj
  }

  fn read_v_or_vn(words: &mut Words) -> VertexCoord {
    Vec3 {
      x: from_str(words.next().unwrap()).unwrap(),
      y: from_str(words.next().unwrap()).unwrap(),
      z: from_str(words.next().unwrap()).unwrap(),
    }
  }

  fn read_vt(words: &mut Words) -> TextureCoord {
    Vec2 {
      x: from_str(words.next().unwrap()).unwrap(),
      y: 1.0 - from_str(words.next().unwrap()).unwrap(), // flip
    }
  }

  fn read_f(words: &mut Words) -> Face {
    let mut face = Face {
      vertex: [0, 0, 0],
      texture: [0, 0, 0],
      normal: [0, 0, 0],
    };
    let mut i = 0;
    for group in *words {
      let mut w = group.split('/');
      face.vertex[i] = from_str(w.next().unwrap()).unwrap();
      face.texture[i] = from_str(w.next().unwrap()).unwrap();
      face.normal[i] = from_str(w.next().unwrap()).unwrap();
      i += 1;
    }
    face
  }

  fn read_line(&mut self, line: &str) {
    let mut words = line.words();
    fn is_correct_tag(tag: &str) -> Bool {
      tag.len() != 0 && tag[0] != ('#' as u8)
    }
    match words.next() {
      Some(tag) if is_correct_tag(tag) => {
        let w = &mut words;
        match tag {
          &"v" => self.coords.push(Model::read_v_or_vn(w)),
          &"vn" => self.normals.push(Model::read_v_or_vn(w)),
          &"vt" => self.texture_coords.push(Model::read_vt(w)),
          &"f" => self.faces.push(Model::read_f(w)),
          _ => {},
        }
      }
      _ => {},
    };
  }

  fn read(&mut self, filename: &str) {
    let path = Path::new(filename);
    let mut file = BufferedReader::new(File::open(&path));
    for line in file.lines() {
      self.read_line(line);
    }
  }

  pub fn build(&self) -> ~[VertexCoord] {
    let mut mesh = ~[];
    for face in self.faces.iter() {
      for i in range(0, 3) {
        let vertex_id = face.vertex[i] - 1;
        mesh.push(self.coords[vertex_id]);
      }
    }
    mesh
  }

  pub fn build_tex_coord(&self) -> ~[TextureCoord] {
    let mut tex_coords = ~[];
    for face in self.faces.iter() {
      for i in range(0, 3) {
        let texture_coord_id = face.texture[i] - 1;
        tex_coords.push(self.texture_coords[texture_coord_id]);
      }
    }
    tex_coords
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
