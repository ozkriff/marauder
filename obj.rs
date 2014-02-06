// See LICENSE file for copyright and license details.

use std::str::Words;
use std::io::{
  BufferedReader,
  File
};
use gl::types::GLfloat;
use cgmath::vector::{
  Vec3,
  Vec2,
};

struct Face {
  vertex: [int, ..3],
  texture: [int, ..3],
  normal: [int, ..3],
}

pub struct Model {
  coords: ~[Vec3<GLfloat>],
  normals: ~[Vec3<GLfloat>],
  texture_coords: ~[Vec2<GLfloat>],
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

  fn read_v_or_vn(words: &mut Words) -> Vec3<GLfloat> {
    Vec3 {
      x: from_str(words.next().unwrap()).unwrap(),
      y: from_str(words.next().unwrap()).unwrap(),
      z: from_str(words.next().unwrap()).unwrap(),
    }
  }

  fn read_vt(words: &mut Words) -> Vec2<GLfloat> {
    // TODO: y = 1.0 - y; // flip vertically
    Vec2 {
      x: from_str(words.next().unwrap()).unwrap(),
      y: from_str(words.next().unwrap()).unwrap(),
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
    fn is_correct_tag(tag: &str) -> bool {
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

  pub fn build(&self) -> ~[Vec3<GLfloat>]{
    let mut mesh = ~[];
    for face in self.faces.iter() {
      for i in range(0, 3) {
        let vertex_id = face.vertex[i] - 1;
        // let texture_coord_id = face.texture[i] - 1; // TODO
        mesh.push(self.coords[vertex_id]);
      }
    }
    mesh
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
