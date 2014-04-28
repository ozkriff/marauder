// See LICENSE file for copyright and license details.

use gl::types::{GLfloat, GLuint};
use cgmath::vector::{Vector3, Vector2};

pub struct Color3 {
    pub r: MFloat,
    pub g: MFloat,
    pub b: MFloat,
}

pub struct Color4 {
    pub r: MFloat,
    pub g: MFloat,
    pub b: MFloat,
    pub a: MFloat,
}

pub type MFloat = GLfloat;

pub type VertexCoord = Vector3<MFloat>;
pub type Normal = Vector3<MFloat>;
pub type TextureCoord = Vector2<MFloat>;

pub struct WorldPos{pub v: Vector3<MFloat>}

pub struct Time{pub n: u64}

pub struct MatId{pub id: GLuint}

pub struct ColorId{pub id: GLuint}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
