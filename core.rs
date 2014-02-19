// See LICENSE file for copyright and license details.

use std::hashmap::HashMap;
use cgmath::vector::Vec2;
use core_types::{
    Size2,
    Int,
    UnitId,
    PlayerId,
    MapPos,
};

pub enum Command {
    CommandMove(UnitId, MapPos),
    CommandEndTurn,
    CommandCreateUnit(MapPos),
}

pub enum Event {
    EventMove(UnitId, ~[MapPos]),
    EventEndTurn(PlayerId, PlayerId), // old_id, new_id
    EventCreateUnit(UnitId, MapPos),
}

pub struct Player {
    id: PlayerId,
}

pub struct Unit {
    id: UnitId,
    pos: MapPos,
}

pub struct Core<'a> {
    units: ~[Unit],
    players: ~[Player],
    current_player_id: PlayerId,
    core_event_list: ~[~CoreEvent],
    event_lists: HashMap<PlayerId, ~[Event]>,
    map_size: Size2<Int>,
}

fn get_event_lists() -> HashMap<PlayerId, ~[Event]> {
    let mut map = HashMap::new();
    map.insert(0 as PlayerId, ~[]);
    map.insert(1 as PlayerId, ~[]);
    map
}

impl<'a> Core<'a> {
    pub fn new() -> Core {
        let mut core = Core {
            units: ~[],
            players: ~[Player{id: 0}, Player{id: 1}],
            current_player_id: 0,
            core_event_list: ~[],
            event_lists: get_event_lists(),
            map_size: Size2{x: 4, y: 8},
        };
        core.do_command(CommandCreateUnit(Vec2{x: 0, y: 0}));
        core.do_command(CommandCreateUnit(Vec2{x: 0, y: 1}));
        core.do_command(CommandCreateUnit(Vec2{x: 2, y: 0}));
        core.do_command(CommandCreateUnit(Vec2{x: 2, y: 2}));
        core
    }

    pub fn get_event(&mut self) -> Option<Event> {
        let list = self.event_lists.get_mut(&self.current_player_id);
        list.shift()
    }

    fn id_to_unit_mut_opt(&'a mut self, id: UnitId) -> Option<&'a mut Unit> {
        self.units.mut_iter().find(|u| u.id == id)
    }

    fn id_to_unit_mut(&'a mut self, id: UnitId) -> &'a mut Unit {
        match self.id_to_unit_mut_opt(id) {
            Some(unit) => unit,
            None => fail!("Bad unit id: {}", id),
        }
    }

    fn id_to_unit_opt(&'a self, id: UnitId) -> Option<&'a Unit> {
        self.units.iter().find(|u| u.id == id)
    }

    fn id_to_unit(&'a self, id: UnitId) -> &'a Unit {
        match self.id_to_unit_opt(id) {
            Some(unit) => unit,
            None => fail!("Bad unit id: {}", id),
        }
    }

    // TODO: Remove or make private
    pub fn unit_at_opt(&'a self, pos: MapPos) -> Option<&'a Unit> {
        self.units.iter().find(|u| u.pos == pos)
    }

    fn command_to_core_event(&self, command: Command) -> ~CoreEvent {
        match command {
            CommandEndTurn => {
                CoreEventEndTurn::new(self) as ~CoreEvent
            },
            CommandCreateUnit(pos) => {
                CoreEventCreateUnit::new(self, pos) as ~CoreEvent
            },
            CommandMove(unit_id, destination) => {
                CoreEventMove::new(self, unit_id, destination) as ~CoreEvent
            },
        }
    }

    pub fn do_command(&mut self, command: Command) {
        let core_event = self.command_to_core_event(command);
        self.core_event_list.push(core_event);
        self.make_events();
    }

    fn make_events(&mut self) {
        while self.core_event_list.len() != 0 {
            let event = self.core_event_list.pop().unwrap();
            event.apply(self);
            for player in self.players.iter() {
                let event_list = self.event_lists.get_mut(&player.id);
                event_list.push(event.to_event());
            }
        }
    }
}

trait CoreEvent {
    fn apply(&self, core: &mut Core);
    fn to_event(&self) -> Event;
    // TODO: fn is_visible(&self) -> Bool;
}

struct CoreEventMove {
    unit_id: UnitId,
    path: ~[MapPos],
}

impl CoreEventMove {
    fn new(
        core: &Core,
        unit_id: UnitId,
        destination: MapPos
    ) -> ~CoreEventMove {
        let start_pos = core.id_to_unit(unit_id).pos;
        ~CoreEventMove {
            path: ~[start_pos, destination],
            unit_id: unit_id,
        }
    }
}

impl CoreEvent for CoreEventMove {
    fn to_event(&self) -> Event {
        EventMove(self.unit_id, self.path.clone())
    }

    fn apply(&self, core: &mut Core) {
        let unit = core.id_to_unit_mut(self.unit_id);
        unit.pos = *self.path.last().unwrap();
    }
}

struct CoreEventEndTurn {
    old_id: PlayerId,
    new_id: PlayerId,
}

impl CoreEventEndTurn {
    fn new(core: &Core) -> ~CoreEventEndTurn {
        let old_id = core.current_player_id;
        let max_id = core.players.len() as Int;
        let new_id = if old_id + 1 == max_id { 0 } else { old_id + 1 };
        ~CoreEventEndTurn{old_id: old_id, new_id: new_id}
    }
}

impl CoreEvent for CoreEventEndTurn {
    fn to_event(&self) -> Event {
        EventEndTurn(self.old_id, self.new_id)
    }

    fn apply(&self, core: &mut Core) {
        // core.deselected_any_units();
        for player in core.players.iter() {
            if player.id == self.new_id {
                if core.current_player_id == self.old_id {
                    core.current_player_id = player.id;
                }
                return;
            }
        }
    }
}

struct CoreEventCreateUnit {
    pos: MapPos,
    id: UnitId,
}

impl CoreEventCreateUnit {
    fn new(core: &Core, pos: MapPos) -> ~CoreEventCreateUnit {
        let last_unit_opt = core.units.last();
        let new_id = if last_unit_opt.is_some() {
            last_unit_opt.unwrap().id + 1
        } else {
            0
        };
        ~CoreEventCreateUnit{id: new_id, pos: pos}
    }
}

impl CoreEvent for CoreEventCreateUnit {
    fn to_event(&self) -> Event {
        EventCreateUnit(self.id, self.pos)
    }

    fn apply(&self, core: &mut Core) {
        assert!(core.units.mut_iter().find(|u| u.id == self.id).is_none());
        core.units.push(Unit{id: self.id, pos: self.pos});
    }
}


// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
