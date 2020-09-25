// See LICENSE file for copyright and license details.

use crate::visualizer::context::Context;
use crate::visualizer::gui::{Button, ButtonId, ButtonManager};
use crate::visualizer::mgl;
use crate::visualizer::state_visualizer::StateChangeCommand::{QuitMenu, StartGame};
use crate::visualizer::state_visualizer::{StateChangeCommand, StateVisualizer};
use crate::visualizer::types::{ScreenPos, Time};
use cgmath::Vector2;
use std::ops::DerefMut;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct MenuStateVisualizer {
    button_manager: ButtonManager,
    button_start_id: ButtonId,
    button_quit_id: ButtonId,
    commands_rx: Receiver<StateChangeCommand>,
    commands_tx: Sender<StateChangeCommand>,
}

impl MenuStateVisualizer {
    pub fn new(context: &Context) -> MenuStateVisualizer {
        let mut button_manager = ButtonManager::new();
        let button_start_id = button_manager.add_button(Button::new(
            "start",
            context.font_stash.borrow_mut().deref_mut(),
            &context.shader,
            ScreenPos {
                v: Vector2 { x: 10, y: 40 },
            },
        ));
        let button_quit_id = button_manager.add_button(Button::new(
            "quit",
            context.font_stash.borrow_mut().deref_mut(),
            &context.shader,
            ScreenPos {
                v: Vector2 { x: 10, y: 10 },
            },
        ));
        let (commands_tx, commands_rx) = channel();
        MenuStateVisualizer {
            button_manager,
            button_start_id,
            button_quit_id,
            commands_rx,
            commands_tx,
        }
    }

    fn handle_mouse_button_event(&mut self, context: &Context) {
        match self.button_manager.get_clicked_button_id(context) {
            Some(button_id) => {
                if button_id == self.button_start_id {
                    self.commands_tx.send(StartGame).unwrap();
                } else if button_id == self.button_quit_id {
                    self.commands_tx.send(QuitMenu).unwrap();
                }
            }
            None => {}
        }
    }
}

impl StateVisualizer for MenuStateVisualizer {
    fn logic(&mut self, _: &Context) {}

    fn draw(&mut self, context: &mut Context, _: Time) {
        mgl::set_clear_color(mgl::BLACK_3);
        mgl::clear_screen();
        context.shader.activate();
        context
            .shader
            .uniform_color(context.basic_color_id, mgl::WHITE);
        self.button_manager.draw(context);
        use glfw::Context;
        context.win.swap_buffers();
    }

    fn handle_event(&mut self, context: &Context, event: glfw::WindowEvent) {
        match event {
            glfw::WindowEvent::Key(key, _, glfw::Action::Press, _) => match key {
                glfw::Key::Kp1 => {
                    self.commands_tx.send(StartGame).unwrap();
                }
                glfw::Key::Escape | glfw::Key::Q => {
                    self.commands_tx.send(QuitMenu).unwrap();
                }
                _ => {}
            },
            glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, glfw::Action::Press, _) => {
                self.handle_mouse_button_event(context);
            }
            _ => {}
        }
    }

    fn get_command(&self) -> Option<StateChangeCommand> {
        match self.commands_rx.try_recv() {
            Ok(cmd) => Some(cmd),
            Err(_) => None,
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
