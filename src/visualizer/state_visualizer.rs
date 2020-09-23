// See LICENSE file for copyright and license details.

use crate::visualizer::types::Time;
use crate::visualizer::context::Context;
use glfw::WindowEvent;

pub enum StateChangeCommand {
    StartGame,
    QuitMenu,
    EndGame,
}

pub trait StateVisualizer {
    fn logic(&mut self, context: &Context);
    fn draw(&mut self, context: &mut Context, dtime: Time);
    // TODO: GLFW context or visualizer context
    fn handle_event(&mut self, context: &Context, event: WindowEvent);
    fn get_command(&self) -> Option<StateChangeCommand>;
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
