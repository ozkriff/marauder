// See LICENSE file for copyright and license details.

use glfw;
use visualizer::types::Time;
use visualizer::context::Context;

pub enum StateChangeCommand {
    StartGame,
    QuitMenu,
    EndGame,
}

pub trait StateVisualizer {
    fn logic(&mut self, context: &Context);
    fn draw(&mut self, context: &Context, dtime: Time);
    fn handle_event(&mut self, context: &Context, event: glfw::WindowEvent);
    fn get_command(&self) -> Option<StateChangeCommand>;
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
