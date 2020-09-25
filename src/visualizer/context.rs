// See LICENSE file for copyright and license details.

use crate::core::conf::Config;
use crate::core::types::{MInt, Size2};
use crate::visualizer::font_stash::FontStash;
use crate::visualizer::mgl;
use crate::visualizer::shader::Shader;
use crate::visualizer::types::{ColorId, MatId, ScreenPos};
use cgmath::Vector2;
use glfw::WindowEvent::{CursorPos, Size};
use std::cell::RefCell;

pub struct Context {
    pub win: glfw::Window,
    pub win_size: Size2<MInt>,
    pub mouse_pos: ScreenPos,
    pub config: Config,
    pub font_stash: RefCell<FontStash>,
    pub shader: Shader,
    pub mvp_mat_id: MatId,
    pub basic_color_id: ColorId,
}

impl Context {
    fn set_window_size(&mut self, win_size: Size2<MInt>) {
        self.win_size = win_size;
        mgl::set_viewport(win_size);
    }

    pub fn handle_event(&mut self, event: glfw::WindowEvent) {
        match event {
            CursorPos(x, y) => {
                self.mouse_pos = ScreenPos {
                    v: Vector2 {
                        x: x as MInt,
                        y: y as MInt,
                    },
                };
            }
            Size(w, h) => {
                self.set_window_size(Size2 { w, h });
            }
            _ => {}
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
