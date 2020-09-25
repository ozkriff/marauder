// See LICENSE file for copyright and license details.

#![warn(non_upper_case_globals)]
#![warn(unused_results)]

extern crate cgmath;
extern crate gl;
extern crate glfw;
extern crate rand;
extern crate stb_image;
extern crate stb_tt;
extern crate time;
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

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
