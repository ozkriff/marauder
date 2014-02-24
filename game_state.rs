// See LICENSE file for copyright and license details.

use collections::hashmap::HashMap;
use core::{
    Unit,
    Event,
    EventMove,
    EventEndTurn,
    EventCreateUnit,
    EventAttackUnit,
};
use core_types::{
    UnitId,
};

pub struct GameState {
    units: HashMap<UnitId, Unit>,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            units: HashMap::new(),
        }
    }

    pub fn apply_event(&mut self, event: &Event) {
        match *event {
            EventMove(id, ref path) => {
                let unit = self.units.get_mut(&id);
                unit.pos = *path.last().unwrap();
            },
            EventEndTurn(_, _) => {},
            EventCreateUnit(id, pos) => {
                assert!(self.units.find(&id).is_none());
                self.units.insert(id, Unit{id: id, pos: pos});
            },
            EventAttackUnit(_, defender_id) => {
                assert!(self.units.find(&defender_id).is_some());
                self.units.remove(&defender_id);
            },
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
