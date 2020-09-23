// See LICENSE file for copyright and license details.

use crate::core::core::{Event, ObjectTypes, Unit};
use crate::core::types::{MapPos, PlayerId, UnitId};
use std::collections::HashMap;

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
                unit.move_points = object_types.get_unit_type(unit.type_id).move_points;
                unit.attacked = false;
            }
        }
    }

    pub fn apply_event(&mut self, object_types: &ObjectTypes, event: &Event) {
        match event {
            Event::EventMove(id, ref path) => {
                let pos = *path.last().unwrap();
                let unit = self.units.get_mut(&id).unwrap();
                unit.pos = pos;
                assert!(unit.move_points > 0);
                unit.move_points = 0;
            }
            Event::EventEndTurn(_, new_player_id) => {
                self.refresh_units(object_types, new_player_id.clone());
            }
            Event::EventCreateUnit(id, pos, type_id, player_id) => {
                assert!(self.units.get(&id).is_none());
                let move_points = object_types.get_unit_type(type_id.clone()).move_points;
                let _ = self.units.insert(
                    id.clone(),
                    Unit {
                        id: id.clone(),
                        pos: pos.clone(),
                        player_id: player_id.clone(),
                        type_id: type_id.clone(),
                        move_points: move_points.clone(),
                        attacked: false,
                    },
                );
            }
            Event::EventAttackUnit(attacker_id, defender_id, killed) => {
                if *killed {
                    assert!(self.units.get(&defender_id).is_some());
                    let _ = self.units.remove(&defender_id).unwrap();
                }
                let unit = self.units.get_mut(&attacker_id).unwrap();
                assert!(!unit.attacked);
                unit.attacked = true;
            }
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
