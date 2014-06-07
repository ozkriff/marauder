// See LICENSE file for copyright and license details.

use std::collections::hashmap::HashMap;
use core::core::{
    Unit,
    Event,
    EventMove,
    EventEndTurn,
    EventCreateUnit,
    EventAttackUnit,
};
use core::types::{UnitId, MapPos};

pub struct GameState {
    pub units: HashMap<UnitId, Unit>,
}

impl<'a> GameState {
    pub fn new() -> GameState {
        GameState {
            units: HashMap::new(),
        }
    }

    pub fn units_at(&'a self, pos: MapPos) -> Vec<&'a Unit> {
        let mut units = Vec::new();
        for (_, unit) in self.units.iter() {
            if unit.pos == pos {
                units.push(unit);
            }
        }
        units
    }

    pub fn apply_event(&mut self, event: &Event) {
        match *event {
            EventMove(id, ref path) => {
                let pos = *path.last().unwrap();
                let unit = self.units.get_mut(&id);
                unit.pos = pos;
            },
            EventEndTurn(_, _) => {},
            EventCreateUnit(id, pos, type_id, player_id) => {
                assert!(self.units.find(&id).is_none());
                self.units.insert(id, Unit {
                    id: id,
                    pos: pos,
                    player_id: player_id,
                    type_id: type_id,
                });
            },
            EventAttackUnit(_, defender_id, killed) => {
                if killed {
                    assert!(self.units.find(&defender_id).is_some());
                    self.units.remove(&defender_id);
                }
            },
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
