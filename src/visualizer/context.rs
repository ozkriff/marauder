// See LICENSE file for copyright and license details.

use std::cell::RefCell;
use glfw;
use cgmath::vector::Vector2;
use core::types::{Size2, MInt};
use core::conf::Config;
use visualizer::types::{MatId, ColorId, ScreenPos};
use visualizer::shader::Shader;
use visualizer::font_stash::FontStash;
use visualizer::mgl;

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
            glfw::CursorPosEvent(x, y) => {
                self.mouse_pos = ScreenPos{v: Vector2 {
                    x: x as MInt,
                    y: y as MInt,
                }};
            },
            glfw::SizeEvent(w, h) => {
                self.set_window_size(Size2{w: w, h: h});
            },
            _ => {},
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
