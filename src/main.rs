// See LICENSE file for copyright and license details.

#![deny(non_camel_case_types)]
#![deny(non_uppercase_statics)]
#![deny(unnecessary_qualification)]
#![deny(unnecessary_typecast)]

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

#[phase(syntax, link)]
extern crate error_context;

use visualizer::visualizer::Visualizer;

mod core;
mod visualizer;

fn main() {
    error_context::ErrorContext::init();
    let mut visualizer = Visualizer::new();
    while visualizer.is_running() {
        visualizer.tick();
    }
}

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
