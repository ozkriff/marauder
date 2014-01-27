// See LICENSE file for copyright and license details.

use std;
use glfw;

pub struct CursorPosEvent {
  x: f32,
  y: f32
}

pub struct CursorPosContext {
  chan: Chan<CursorPosEvent>
}

impl glfw::CursorPosCallback for CursorPosContext {
  fn call(&self, _: &glfw::Window, xpos: f64, ypos: f64) {
    self.chan.send(CursorPosEvent {
      x: xpos as f32,
      y: ypos as f32
    });
  }
}

pub struct KeyEvent {
  key: glfw::Key,
  action: glfw::Action
}

pub struct KeyContext {
  chan: Chan<KeyEvent>
}

impl glfw::KeyCallback for KeyContext {
  fn call(
    &self,
    _:      &glfw::Window,
    key:    glfw::Key,
    _:      std::libc::c_int,
    action: glfw::Action,
    _:      glfw::Modifiers
  ) {
    self.chan.send(KeyEvent {
      key: key,
      action: action
    });
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
