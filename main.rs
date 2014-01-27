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

use visualizer::Visualizer;

mod misc;
mod visualizer;
mod gl_helpers;
mod glfw_callacks;
mod camera;

fn main() {
  let mut visualizer = Visualizer::new();
  while visualizer.is_running() {
    visualizer.process_events();
    visualizer.pick_tile();
    visualizer.draw();
  }
}

#[start]
fn start(argc: int, argv: **u8) -> int {
  native::start(argc, argv, main)
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
