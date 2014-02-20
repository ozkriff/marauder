// See LICENSE file for copyright and license details.

use core::{
    Unit,
};

pub struct GameState {
    units: ~[Unit],
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            units: ~[],
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
