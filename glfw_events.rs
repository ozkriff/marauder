// See LICENSE file for copyright and license details.

use std::comm::{
  Port,
  Chan,
  Data
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

pub struct EventHandler<T> {
  port: Port<T>,
}

impl<T: Send> EventHandler<T> {
  pub fn new() -> (EventHandler<T>, Chan<T>) {
    let (port, chan) = Chan::new();
    (EventHandler{port: port}, chan)
  }

  pub fn handle(&self, f: |T|) {
    loop {
      match self.port.try_recv() {
        Data(e) => f(e),
        _ => break
      }
    }
  }
}

pub struct EventHandlers {
  key_handler: EventHandler<KeyEvent>,
  cursor_pos_handler: EventHandler<CursorPosEvent>,
}

impl EventHandlers {
  pub fn new(win: &glfw::Window) -> EventHandlers {
    let (key_handler, key_event_chan) = EventHandler::new();
    let (cursor_pos_handler, cursor_pos_chan) = EventHandler::new();
    win.set_key_callback(
      ~KeyContext{chan: key_event_chan});
    win.set_cursor_pos_callback(
      ~CursorPosContext{chan: cursor_pos_chan});
    EventHandlers {
      key_handler: key_handler,
      cursor_pos_handler: cursor_pos_handler,
    }
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
