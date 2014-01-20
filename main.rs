// See LICENSE file for copyright and license details.

extern mod native;
extern mod win;

use win::Win;

fn main() {
  let mut win = Win::new();
  while win.is_running() {
    win.process_events();
    win.draw();
  }
}

#[start]
fn start(argc: int, argv: **u8) -> int {
  native::start(argc, argv, main)
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
