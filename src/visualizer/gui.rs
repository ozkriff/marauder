// See LICENSE file for copyright and license details.

use collections::hashmap::HashMap;
use core::types::{MInt, Size2, Point2};
use visualizer::shader::Shader;
use visualizer::font_stash::FontStash;

#[deriving(Eq, TotalEq, Hash)]
pub struct ButtonId {pub id: MInt}

pub struct Button {
    pos: Point2<MInt>, // TODO: ScreenPos
    size: Size2<MInt>,
    label: StrBuf,
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
            label: StrBuf::from_str(label),
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
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
