// See LICENSE file for copyright and license details.

use std::hashmap::HashMap;
use gl::types::GLfloat;
use cgmath::vector::{
  Vec3,
  Vec2,
};
use core_types::Int;

pub struct Color3 {
  r: GLfloat,
  g: GLfloat,
  b: GLfloat,
}

pub type Float = GLfloat; // TODO: rename, collision with trait
pub type WorldPos = Vec3<Float>;
pub type VertexCoord = Vec3<Float>;
pub type Normal = Vec3<Float>;
pub type TextureCoord = Vec2<Float>;
pub type Point2<T> = Vec2<T>;
// TODO: pub type Handle = GLuint;

pub struct SceneNode {
  pos: WorldPos,
  // rot: Angle,
}

pub type NodeId = Int;
pub type Scene = HashMap<NodeId, SceneNode>;

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
