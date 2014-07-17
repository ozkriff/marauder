// See LICENSE file for copyright and license details.

use std::collections::hashmap::HashMap;
use cgmath::vector::Vector3;
use core::types::{MInt, Size2};
use visualizer::types::{MFloat, ScreenPos};
use visualizer::shader::Shader;
use visualizer::font_stash::FontStash;
use visualizer::context::Context;
use visualizer::mesh::Mesh;
use visualizer::mgl;

#[deriving(PartialEq, Eq, Hash)]
pub struct ButtonId {pub id: MInt}

pub struct Button {
    pos: ScreenPos,
    size: Size2<MInt>,
    mesh: Mesh,
}

impl Button {
    pub fn new(
        label: &str,
        font_stash: &mut FontStash,
        shader: &Shader,
        pos: ScreenPos
    ) -> Button {
        let (_, size) = font_stash.get_text_size(label);
        Button {
            pos: pos,
            size: size,
            mesh: font_stash.get_mesh(label, shader),
        }
    }

    pub fn draw(&self, shader: &Shader) {
        self.mesh.draw(shader);
    }

    pub fn pos(&self) -> ScreenPos {
        self.pos
    }

    pub fn size(&self) -> Size2<MInt> {
        self.size
    }
}

pub struct ButtonManager {
    buttons: HashMap<ButtonId, Button>,
    last_id: ButtonId,
}

impl ButtonManager {
    pub fn new() -> ButtonManager {
        ButtonManager {
            buttons: HashMap::new(),
            last_id: ButtonId{id: 0},
        }
    }

    pub fn buttons<'a>(&'a self) -> &'a HashMap<ButtonId, Button> {
        &self.buttons
    }

    pub fn add_button(&mut self, button: Button) -> ButtonId {
        let id = self.last_id;
        self.buttons.insert(id, button);
        self.last_id.id += 1;
        id
    }

    pub fn get_clicked_button_id(&self, context: &Context) -> Option<ButtonId> {
        let x = context.mouse_pos.v.x;
        let y = context.win_size.h - context.mouse_pos.v.y;
        for (id, button) in self.buttons().iter() {
            if x >= button.pos().v.x
                && x <= button.pos().v.x + button.size().w
                && y >= button.pos().v.y
                && y <= button.pos().v.y + button.size().h
            {
                return Some(*id);
            }
        }
        None
    }

    pub fn draw(&self, context: &Context) {
        let m = mgl::get_2d_screen_matrix(context.win_size);
        for (_, button) in self.buttons().iter() {
            let text_offset = Vector3 {
                x: button.pos().v.x as MFloat,
                y: button.pos().v.y as MFloat,
                z: 0.0,
            };
            context.shader.uniform_mat4f(
                context.mvp_mat_id, &mgl::tr(m, text_offset));
            button.draw(&context.shader);
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
