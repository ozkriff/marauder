// See LICENSE file for copyright and license details.

use crate::core::types::MInt;
use cgmath::{Vector2, Vector3};
use gl::types::{GLfloat, GLuint};

pub type MFloat = GLfloat;

#[derive(Copy, Clone)]
pub struct Color3 {
    pub r: MFloat,
    pub g: MFloat,
    pub b: MFloat,
}

#[derive(Copy, Clone)]
pub struct Color4 {
    pub r: MFloat,
    pub g: MFloat,
    pub b: MFloat,
    pub a: MFloat,
}

#[derive(Copy, Clone)]
pub struct VertexCoord {
    pub v: Vector3<MFloat>,
}

#[derive(Copy, Clone)]
pub struct Normal {
    pub v: Vector3<MFloat>,
}

#[derive(Copy, Clone)]
pub struct TextureCoord {
    pub v: Vector2<MFloat>,
}

#[derive(Clone, Copy)]
pub struct WorldPos {
    pub v: Vector3<MFloat>,
}

#[derive(Copy, Clone)]
pub struct ScreenPos {
    pub v: Vector2<MInt>,
}

#[derive(Copy, Clone)]
pub struct Time {
    pub n: u64,
}

#[derive(Copy, Clone)]
pub struct MatId {
    pub id: GLuint,
}

#[derive(Copy, Clone)]
pub struct ColorId {
    pub id: GLuint,
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
