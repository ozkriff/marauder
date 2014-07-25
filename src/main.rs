// See LICENSE file for copyright and license details.

#![warn(unnecessary_qualification)]
#![warn(unnecessary_typecast)]
#![warn(non_uppercase_statics)]
#![warn(unused_result)]

#![feature(macro_rules)]
#![feature(phase)]

extern crate native;
extern crate serialize;
extern crate collections;
extern crate time;
extern crate rand;
extern crate cgmath;
extern crate glfw;
extern crate gl;
extern crate stb_image;
extern crate stb_tt;

#[phase(plugin, link)]
extern crate error_context;

use visualizer::visualizer::Visualizer;

mod core;
mod visualizer;

fn main() {
    let mut visualizer = Visualizer::new();
    while visualizer.is_running() {
        visualizer.tick();
    }
}

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
