// See LICENSE file for copyright and license details.

use std::collections::hashmap::HashMap;
use core::core::{
    ObjectTypes,
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

    fn refresh_units(&mut self, object_types: &ObjectTypes, player_id: PlayerId) {
        for (_, unit) in self.units.iter_mut() {
            if unit.player_id == player_id {
                unit.move_points
                    = object_types.get_unit_type(unit.type_id).move_points;
                unit.attacked = false;
            }
        }
    }

    pub fn apply_event(&mut self, object_types: &ObjectTypes, event: &Event) {
        match *event {
            EventMove(id, ref path) => {
                let pos = *path.last().unwrap();
                let unit = self.units.get_mut(&id);
                unit.pos = pos;
                assert!(unit.move_points > 0);
                unit.move_points = 0;
            },
            EventEndTurn(_, new_player_id) => {
                self.refresh_units(object_types, new_player_id);
            },
            EventCreateUnit(id, pos, type_id, player_id) => {
                assert!(self.units.find(&id).is_none());
                let move_points
                    = object_types.get_unit_type(type_id).move_points;
                self.units.insert(id, Unit {
                    id: id,
                    pos: pos,
                    player_id: player_id,
                    type_id: type_id,
                    move_points: move_points,
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
