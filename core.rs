// See LICENSE file for copyright and license details.

use collections::hashmap::HashMap;
use cgmath::vector::Vec2;
use core_types::{
    Size2,
    Int,
    UnitId,
    PlayerId,
    MapPos,
};

pub enum Command {
    CommandMove(UnitId, ~[MapPos]),
    CommandEndTurn,
    CommandCreateUnit(MapPos),
    CommandAttackUnit(UnitId, UnitId),
}

pub enum Event {
    EventMove(UnitId, ~[MapPos]),
    EventEndTurn(PlayerId, PlayerId), // old_id, new_id
    EventCreateUnit(UnitId, MapPos),
    EventAttackUnit(UnitId, UnitId),
}

pub struct Player {
    id: PlayerId,
}

pub struct Unit {
    id: UnitId,
    pos: MapPos,
}

pub struct Core<'a> {
    priv units: HashMap<UnitId, Unit>,
    priv players: ~[Player],
    priv current_player_id: PlayerId,
    priv core_event_list: ~[~CoreEvent],
    priv event_lists: HashMap<PlayerId, ~[Event]>,
    priv map_size: Size2<Int>,
}

fn get_event_lists() -> HashMap<PlayerId, ~[Event]> {
    let mut map = HashMap::new();
    map.insert(PlayerId(0), ~[]);
    map.insert(PlayerId(1), ~[]);
    map
}

impl<'a> Core<'a> {
    pub fn new() -> ~Core {
        let mut core = ~Core {
            units: HashMap::new(),
            players: ~[Player{id: PlayerId(0)}, Player{id: PlayerId(1)}],
            current_player_id: PlayerId(0),
            core_event_list: ~[],
            event_lists: get_event_lists(),
            map_size: Size2{w: 4, h: 8}, // TODO: Read from json config
        };
        core.do_command(CommandCreateUnit(Vec2{x: 0, y: 0}));
        core.do_command(CommandCreateUnit(Vec2{x: 0, y: 1}));
        core.do_command(CommandCreateUnit(Vec2{x: 2, y: 0}));
        core.do_command(CommandCreateUnit(Vec2{x: 2, y: 2}));
        core
    }

    pub fn map_size(&self) -> Size2<Int> {
        self.map_size
    }

    pub fn player_id(&self) -> PlayerId {
        self.current_player_id
    }

    pub fn get_event(&mut self) -> Option<Event> {
        let list = self.event_lists.get_mut(&self.current_player_id);
        list.shift()
    }

    fn command_to_core_event(&self, command: Command) -> ~CoreEvent {
        match command {
            CommandEndTurn => {
                CoreEventEndTurn::new(self) as ~CoreEvent
            },
            CommandCreateUnit(pos) => {
                CoreEventCreateUnit::new(self, pos) as ~CoreEvent
            },
            CommandMove(unit_id, path) => {
                CoreEventMove::new(self, unit_id, path) as ~CoreEvent
            },
            CommandAttackUnit(attacker_id, defender_id) => {
                CoreEventAttackUnit::new(self, attacker_id, defender_id) as ~CoreEvent
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
    fn new(_: &Core, unit_id: UnitId, path: ~[MapPos]) -> ~CoreEventMove {
        ~CoreEventMove {
            path: path,
            unit_id: unit_id,
        }
    }
}

impl CoreEvent for CoreEventMove {
    fn to_event(&self) -> Event {
        EventMove(self.unit_id, self.path.clone())
    }

    fn apply(&self, core: &mut Core) {
        let unit = core.units.get_mut(&self.unit_id);
        unit.pos = *self.path.last().unwrap();
    }
}

struct CoreEventEndTurn {
    old_id: PlayerId,
    new_id: PlayerId,
}

impl CoreEventEndTurn {
    fn new(core: &Core) -> ~CoreEventEndTurn {
        let PlayerId(old_id) = core.current_player_id;
        let max_id = core.players.len() as Int;
        let new_id = if old_id + 1 == max_id { 0 } else { old_id + 1 };
        ~CoreEventEndTurn{old_id: PlayerId(old_id), new_id: PlayerId(new_id)}
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
        let new_id = match core.units.keys().max_by(|&n| n) {
            Some(n) => {
                let UnitId(id) = *n;
                id + 1
            },
            None => 0,
        };
        ~CoreEventCreateUnit{id: UnitId(new_id), pos: pos}
    }
}

impl CoreEvent for CoreEventCreateUnit {
    fn to_event(&self) -> Event {
        EventCreateUnit(self.id, self.pos)
    }

    fn apply(&self, core: &mut Core) {
        assert!(core.units.find(&self.id).is_none());
        core.units.insert(self.id, Unit{id: self.id, pos: self.pos});
    }
}

struct CoreEventAttackUnit {
    attacker_id: UnitId,
    defender_id: UnitId,
}

impl CoreEventAttackUnit {
    fn new(
        _: &Core,
        attacker_id: UnitId,
        defender_id: UnitId
    ) -> ~CoreEventAttackUnit {
        ~CoreEventAttackUnit {
            attacker_id: attacker_id,
            defender_id: defender_id,
        }
    }
}

impl CoreEvent for CoreEventAttackUnit {
    fn to_event(&self) -> Event {
        EventAttackUnit(self.attacker_id, self.defender_id)
    }

    fn apply(&self, core: &mut Core) {
        assert!(core.units.find(&self.defender_id).is_some());
        core.units.remove(&self.defender_id);
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
