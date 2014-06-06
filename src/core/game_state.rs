// See LICENSE file for copyright and license details.

use std::collections::hashmap::HashMap;
use core::core::{
    Unit,
    Event,
    EventMove,
    EventEndTurn,
    EventCreateUnit,
    EventAttackUnit,
    SLOTS_COUNT,
};
use core::types::{UnitId, SlotId, MapPos};

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
                let slot_id = self.get_free_slot(id, pos).unwrap();
                let unit = self.units.get_mut(&id);
                unit.pos = pos;
                unit.slot_id = slot_id;
            },
            EventEndTurn(_, _) => {},
            EventCreateUnit(id, pos, type_id, player_id) => {
                assert!(self.units.find(&id).is_none());
                let slot_id = self.get_free_slot(id, pos).unwrap();
                self.units.insert(id, Unit {
                    id: id,
                    pos: pos,
                    slot_id: slot_id,
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

    // TODO: simplify
    pub fn get_free_slot(&self, unit_id: UnitId, pos: MapPos) -> Option<SlotId> {
        for id in range(0, SLOTS_COUNT) {
            let mut index = Some(SlotId{id: id});
            for unit in self.units_at(pos).iter() {
                if unit.id == unit_id {
                    return Some(unit.slot_id);
                }
                if unit.slot_id.id == id {
                    index = None;
                    break;
                }
            }
            if index.is_some() {
                return index;
            }
        }
        None
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
