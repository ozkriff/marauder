// See LICENSE file for copyright and license details.

use std::comm::{
  Port,
  Chan
};
use std;
use glfw;

pub struct CursorPosEvent {
  x: f32,
  y: f32
}

struct CursorPosContext {
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

struct KeyContext {
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

pub struct EventPorts {
  key_event_port: Port<KeyEvent>,
  cursor_pos_event_port: Port<CursorPosEvent>,
}

impl EventPorts {
  pub fn new(glfw_win: &glfw::Window) -> EventPorts {
    let (key_event_port, key_event_chan) = Chan::new();
    let (cursor_pos_event_port, cursor_pos_chan) = Chan::new();

    let event_ports = EventPorts {
      key_event_port: key_event_port,
      cursor_pos_event_port: cursor_pos_event_port,
    };

    glfw_win.set_key_callback(
      ~KeyContext{chan: key_event_chan});
    glfw_win.set_cursor_pos_callback(
      ~CursorPosContext{chan: cursor_pos_chan});

    event_ports
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
