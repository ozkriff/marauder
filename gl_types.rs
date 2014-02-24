// See LICENSE file for copyright and license details.

use collections::hashmap::HashMap;
use gl::types::{
    GLfloat,
    GLuint,
};
use cgmath::vector::{
    Vec3,
    Vec2,
};
use core_types::Int;

pub struct Color3 {
    r: Float,
    g: Float,
    b: Float,
}

pub type Float = GLfloat; // TODO: rename, collision with trait
pub type WorldPos = Vec3<Float>;
pub type VertexCoord = Vec3<Float>;
pub type Normal = Vec3<Float>;
pub type TextureCoord = Vec2<Float>;
pub type Point2<T> = Vec2<T>;

pub struct ShaderId(GLuint);
pub struct TextureId(GLuint);
pub struct VboId(GLuint);
pub struct AttrId(GLuint);
pub struct MatId(GLuint);

pub struct SceneNode {
    pos: WorldPos,
    // rot: Angle,
}

#[deriving(Ord, Eq, Hash)]
pub struct NodeId(Int);

pub type Scene = HashMap<NodeId, SceneNode>;

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
