// See LICENSE file for copyright and license details.

use collections::hashmap::HashMap;
use cgmath::vector::Vec2;
use core::types::{Size2, MInt, UnitId, PlayerId, MapPos};
use core::conf::Config;

pub enum Command {
    CommandMove(UnitId, Vec<MapPos>),
    CommandEndTurn,
    CommandCreateUnit(MapPos),
    CommandAttackUnit(UnitId, UnitId),
}

pub enum Event {
    EventMove(UnitId, Vec<MapPos>),
    EventEndTurn(PlayerId, PlayerId), // old_id, new_id
    EventCreateUnit(UnitId, MapPos, PlayerId),
    EventAttackUnit(UnitId, UnitId),
}

pub struct Player {
    pub id: PlayerId,
}

pub struct Unit {
    pub id: UnitId,
    pub pos: MapPos,
    pub player_id: PlayerId,
}

pub struct Core {
    units: HashMap<UnitId, Unit>,
    players: Vec<Player>,
    current_player_id: PlayerId,
    core_event_list: Vec<~CoreEvent>,
    event_lists: HashMap<PlayerId, Vec<Event>>,
    map_size: Size2<MInt>,
}

fn get_event_lists() -> HashMap<PlayerId, Vec<Event>> {
    let mut map = HashMap::new();
    map.insert(PlayerId(0), Vec::new());
    map.insert(PlayerId(1), Vec::new());
    map
}

impl Core {
    pub fn new() -> ~Core {
        let config = Config::new("conf_core.json");
        let map_size = config.get("map_size");
        let mut core = ~Core {
            units: HashMap::new(),
            players: vec!(Player{id: PlayerId(0)}, Player{id: PlayerId(1)}),
            current_player_id: PlayerId(0),
            core_event_list: Vec::new(),
            event_lists: get_event_lists(),
            map_size: map_size,
        };
        core.add_unit(Vec2{x: 0, y: 0}, PlayerId(0));
        core.add_unit(Vec2{x: 0, y: 1}, PlayerId(0));
        core.add_unit(Vec2{x: 2, y: 0}, PlayerId(1));
        core.add_unit(Vec2{x: 2, y: 2}, PlayerId(1));
        core
    }

    fn add_unit(&mut self, pos: MapPos, player_id: PlayerId) {
        let core_event = CoreEventCreateUnit::new(
            self, pos, player_id);
        self.do_core_event(core_event);
    }

    pub fn map_size(&self) -> Size2<MInt> {
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
                CoreEventCreateUnit::new(
                    self,
                    pos,
                    self.current_player_id,
                ) as ~CoreEvent
            },
            CommandMove(unit_id, path) => {
                CoreEventMove::new(self, unit_id, path) as ~CoreEvent
            },
            CommandAttackUnit(attacker_id, defender_id) => {
                CoreEventAttackUnit::new(
                    self,
                    attacker_id,
                    defender_id,
                ) as ~CoreEvent
            },
        }
    }

    pub fn do_command(&mut self, command: Command) {
        let core_event = self.command_to_core_event(command);
        self.do_core_event(core_event);
    }

    fn do_core_event(&mut self, core_event: ~CoreEvent) {
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
    // TODO: fn is_visible(&self) -> MBool;
}

struct CoreEventMove {
    unit_id: UnitId,
    path: Vec<MapPos>,
}

impl CoreEventMove {
    fn new(_: &Core, unit_id: UnitId, path: Vec<MapPos>) -> ~CoreEventMove {
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
        let max_id = core.players.len() as MInt;
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
    player_id: PlayerId,
}

impl CoreEventCreateUnit {
    fn new(
        core: &Core,
        pos: MapPos,
        player_id: PlayerId
    ) -> ~CoreEventCreateUnit {
        let new_id = match core.units.keys().max_by(|&n| n) {
            Some(n) => {
                let UnitId(id) = *n;
                id + 1
            },
            None => 0,
        };
        ~CoreEventCreateUnit {
            id: UnitId(new_id),
            pos: pos,
            player_id: player_id,
        }
    }
}

impl CoreEvent for CoreEventCreateUnit {
    fn to_event(&self) -> Event {
        EventCreateUnit(self.id, self.pos, self.player_id)
    }

    fn apply(&self, core: &mut Core) {
        assert!(core.units.find(&self.id).is_none());
        core.units.insert(self.id, Unit {
            id: self.id,
            pos: self.pos,
            player_id: core.current_player_id,
        });
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
