// See LICENSE file for copyright and license details.

use core::{
    Unit,
    Event,
    EventMove,
    EventEndTurn,
    EventCreateUnit,
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

    pub fn apply_event(&mut self, event: &Event) {
        match *event {
            EventMove(id, ref path) => {
                let unit = self.units.mut_iter().find(|u| u.id == id).unwrap();
                unit.pos = *path.last().unwrap();
            },
            EventEndTurn(_, _) => {},
            EventCreateUnit(id, pos) => {
                assert!(self.units.iter().find(|u| u.id == id).is_none());
                self.units.push(Unit{id: id, pos: pos});
            },
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
