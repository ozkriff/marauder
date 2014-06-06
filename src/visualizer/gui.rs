// See LICENSE file for copyright and license details.

use std::collections::hashmap::HashMap;
use core::types::{MInt, Size2, Point2};
use visualizer::shader::Shader;
use visualizer::font_stash::FontStash;
use visualizer::context::Context;

#[deriving(PartialEq, Eq, Hash)]
pub struct ButtonId {pub id: MInt}

pub struct Button {
    pos: Point2<MInt>, // TODO: ScreenPos
    size: Size2<MInt>,
    label: String,
}

impl Button {
    pub fn new(
        label: &str,
        font_stash: &mut FontStash,
        pos: Point2<MInt>
    ) -> Button {
        let (_, size) = font_stash.get_text_size(label);
        Button {
            pos: pos,
            size: size,
            label: String::from_str(label),
        }
    }

    pub fn draw(&self, font_stash: &mut FontStash, shader: &Shader) {
        let text_mesh = font_stash.get_mesh(self.label.as_slice(), shader);
        text_mesh.draw(shader);
    }

    pub fn pos(&self) -> Point2<MInt> {
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
        let x = context.mouse_pos.v.x as MInt;
        let y = context.win_size.h - context.mouse_pos.v.y as MInt;
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
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
