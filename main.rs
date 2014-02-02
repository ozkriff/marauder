// See LICENSE file for copyright and license details.

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
mod glfw_events;
mod camera;
mod map;
mod color;
mod geom;
mod tile_picker;

fn main() {
  let mut visualizer = Visualizer::new();
  while visualizer.is_running() {
    visualizer.handle_events();
    visualizer.pick_tile();
    visualizer.draw();
  }
}

#[start]
fn start(argc: int, argv: **u8) -> int {
  native::start(argc, argv, main)
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
