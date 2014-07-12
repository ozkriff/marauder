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
use core::types::{PlayerId, UnitId, MapPos};

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

    fn refresh_units(&mut self, player_id: PlayerId) {
        for (_, unit) in self.units.mut_iter() {
            if unit.player_id == player_id {
                unit.moved = false;
                unit.attacked = false;
            }
        }
    }

    pub fn apply_event(&mut self, event: &Event) {
        match *event {
            EventMove(id, ref path) => {
                let pos = *path.last().unwrap();
                let unit = self.units.get_mut(&id);
                unit.pos = pos;
                assert!(!unit.moved);
                unit.moved = true;
            },
            EventEndTurn(_, new_player_id) => {
                self.refresh_units(new_player_id);
            },
            EventCreateUnit(id, pos, type_id, player_id) => {
                assert!(self.units.find(&id).is_none());
                self.units.insert(id, Unit {
                    id: id,
                    pos: pos,
                    player_id: player_id,
                    type_id: type_id,
                    moved: false,
                    attacked: false,
                });
            },
            EventAttackUnit(attacker_id, defender_id, killed) => {
                if killed {
                    assert!(self.units.find(&defender_id).is_some());
                    self.units.remove(&defender_id);
                }
                let unit = self.units.get_mut(&attacker_id);
                assert!(!unit.attacked);
                unit.attacked = true;
            },
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
