// See LICENSE file for copyright and license details.

// Marauder is turn-based strategy game with hex grid.

#[deny(non_camel_case_types)];
#[deny(non_uppercase_statics)];
#[deny(unnecessary_qualification)];
#[deny(unnecessary_typecast)];

extern mod native;
extern mod cgmath;
extern mod glfw;
extern mod gl;

use win::Win;

mod misc;
mod win;

fn main() {
  let mut win = Win::new();
  while win.is_running() {
    win.process_events();
    win.pick_tile();
    win.draw();
  }
}

#[start]
fn start(argc: int, argv: **u8) -> int {
  native::start(argc, argv, main)
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
