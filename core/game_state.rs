// See LICENSE file for copyright and license details.

use collections::hashmap::HashMap;
use core::core::{
    Unit,
    Event,
    EventMove,
    EventEndTurn,
    EventCreateUnit,
    EventAttackUnit,
};
use core::types::{UnitId, MapPos, MInt};

pub struct GameState {
    units: HashMap<UnitId, Unit>,
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
                let unit = self.units.get_mut(&id);
                unit.pos = *path.last().unwrap();
            },
            EventEndTurn(_, _) => {},
            EventCreateUnit(id, pos, player_id) => {
                assert!(self.units.find(&id).is_none());
                self.units.insert(id, Unit {
                    id: id,
                    pos: pos,
                    player_id: player_id,
                });
            },
            EventAttackUnit(_, defender_id) => {
                assert!(self.units.find(&defender_id).is_some());
                self.units.remove(&defender_id);
            },
        }
    }

    pub fn get_slot_index(&self, unit_id: UnitId, pos: MapPos) -> MInt {
        let mut index = 0;
        for unit in self.units_at(pos).iter() {
            if unit.id == unit_id {
                break;
            }
            index += 1;
        }
        index
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
