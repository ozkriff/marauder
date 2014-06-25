// See LICENSE file for copyright and license details.

use glfw;
use cgmath::vector::Vector2;
use core::types::{Point2};
use visualizer::mgl;
use visualizer::types::Time;
use visualizer::gui::{ButtonManager, Button, ButtonId};
use visualizer::context::Context;
use visualizer::state_visualizer::{
    StateVisualizer,
    StateChangeCommand,
    StartGame,
    QuitMenu,
};

pub struct MenuStateVisualizer {
    button_manager: ButtonManager,
    button_start_id: ButtonId,
    button_quit_id: ButtonId,
    commands: Vec<StateChangeCommand>,
}

impl MenuStateVisualizer {
    pub fn new(context: &Context) -> MenuStateVisualizer {
        let mut button_manager = ButtonManager::new();
        let button_start_id = button_manager.add_button(Button::new(
            "start",
            context.font_stash.borrow_mut().deref_mut(),
            &context.shader,
            Point2{v: Vector2{x: 10, y: 40}})
        );
        let button_quit_id = button_manager.add_button(Button::new(
            "quit",
            context.font_stash.borrow_mut().deref_mut(),
            &context.shader,
            Point2{v: Vector2{x: 10, y: 10}})
        );
        MenuStateVisualizer {
            commands: Vec::new(),
            button_manager: button_manager,
            button_start_id: button_start_id,
            button_quit_id: button_quit_id,
        }
    }

    fn handle_mouse_button_event(&mut self, context: &Context) {
        match self.button_manager.get_clicked_button_id(context) {
            Some(button_id) => {
                if button_id == self.button_start_id {
                    self.commands.push(StartGame);
                } else if button_id == self.button_quit_id {
                    self.commands.push(QuitMenu);
                }
            },
            None => {},
        }
    }
}

impl StateVisualizer for MenuStateVisualizer {
    fn logic(&mut self) {}

    fn draw(&mut self, context: &Context, _: Time) {
        use glfw::Context;
        mgl::set_clear_color(mgl::BLACK_3);
        mgl::clear_screen();
        context.shader.activate();
        context.shader.uniform_color(context.basic_color_id, mgl::WHITE);
        self.button_manager.draw(context);
        context.win.swap_buffers();
    }

    fn handle_event(&mut self, context: &Context, event: glfw::WindowEvent) {
        match event {
            glfw::KeyEvent(key, _, glfw::Press, _) => {
                match key {
                    glfw::Key1 => {
                        self.commands.push(StartGame);
                    },
                    glfw::KeyEscape | glfw::KeyQ => {
                        self.commands.push(QuitMenu);
                    },
                    _ => {},
                }
            },
            glfw::MouseButtonEvent(glfw::MouseButtonLeft, glfw::Press, _) => {
                self.handle_mouse_button_event(context);
            },
            _ => {},
        }
    }

    fn get_command(&mut self) -> Option<StateChangeCommand> {
        self.commands.pop()
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
