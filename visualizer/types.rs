// See LICENSE file for copyright and license details.

use collections::hashmap::HashMap;
use gl::types::{GLfloat, GLuint};
use cgmath::vector::{Vec3, Vec2};
use core::types::MInt;

pub struct Color3 {
    pub r: MFloat,
    pub g: MFloat,
    pub b: MFloat,
}

pub type MFloat = GLfloat;
pub type WorldPos = Vec3<MFloat>;
pub type VertexCoord = Vec3<MFloat>;
pub type Normal = Vec3<MFloat>;
pub type TextureCoord = Vec2<MFloat>;
pub type Point2<T> = Vec2<T>;
pub type Time = u64;

pub struct MatId(GLuint);

pub struct SceneNode {
    pub pos: WorldPos,
    pub rot: MFloat,
    pub mesh_id: MInt,
}

#[deriving(Ord, Eq, TotalEq, Hash)]
pub struct NodeId(MInt);

pub type Scene = HashMap<NodeId, SceneNode>;

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
