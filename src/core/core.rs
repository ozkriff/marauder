// See LICENSE file for copyright and license details.

use std::rand::{task_rng, Rng};
use collections::hashmap::HashMap;
use cgmath::vector::Vector2;
use error_context;
use core::types::{Size2, MInt, UnitId, SlotId, PlayerId, MapPos};
use core::conf::Config;
use core::game_state::GameState;
use core::fs::FileSystem;

pub static SLOTS_COUNT: MInt = 4;

pub enum Command {
    CommandMove(UnitId, Vec<MapPos>),
    CommandEndTurn,
    CommandCreateUnit(MapPos),
    CommandAttackUnit(UnitId, UnitId),
}

pub enum Event {
    EventMove(UnitId, Vec<MapPos>),
    EventEndTurn(PlayerId, PlayerId), // old_id, new_id
    EventCreateUnit(UnitId, MapPos, UnitTypeId, PlayerId),
    EventAttackUnit(UnitId, UnitId, /* killed: */ bool),
}

pub struct Player {
    pub id: PlayerId,
}

pub enum UnitTypeId {
    Tank,
    Soldier,
}

pub struct Unit {
    pub id: UnitId,
    pub pos: MapPos,
    pub slot_id: SlotId,
    pub player_id: PlayerId,
    pub type_id: UnitTypeId,
}

pub struct Core {
    game_state: GameState,
    players: Vec<Player>,
    current_player_id: PlayerId,
    core_event_list: Vec<Box<CoreEvent>>,
    event_lists: HashMap<PlayerId, Vec<Event>>,
    map_size: Size2<MInt>,
}

fn get_event_lists() -> HashMap<PlayerId, Vec<Event>> {
    let mut map = HashMap::new();
    map.insert(PlayerId{id: 0}, Vec::new());
    map.insert(PlayerId{id: 1}, Vec::new());
    map
}

fn get_players_list() -> Vec<Player> {
    vec!(
        Player{id: PlayerId{id: 0}},
        Player{id: PlayerId{id: 1}},
    )
}

impl Core {
    pub fn new(fs: &FileSystem) -> Core {
        set_error_context!("constructing Core", "-");
        let config = Config::new(&fs.get(&Path::new("data/conf_core.json")));
        let map_size = config.get("map_size");
        let mut core = Core {
            game_state: GameState::new(),
            players: get_players_list(),
            current_player_id: PlayerId{id: 0},
            core_event_list: Vec::new(),
            event_lists: get_event_lists(),
            map_size: map_size,
        };
        core.add_unit(MapPos{v: Vector2{x: 0, y: 0}}, Tank, PlayerId{id: 0});
        core.add_unit(MapPos{v: Vector2{x: 0, y: 1}}, Soldier, PlayerId{id: 0});
        core.add_unit(MapPos{v: Vector2{x: 2, y: 0}}, Tank, PlayerId{id: 1});
        core.add_unit(MapPos{v: Vector2{x: 2, y: 2}}, Soldier, PlayerId{id: 1});
        core
    }

    fn add_unit(&mut self, pos: MapPos, type_id: UnitTypeId, player_id: PlayerId) {
        let core_event = CoreEventCreateUnit::new(
            self, pos, type_id, player_id);
        self.do_core_event(core_event);
    }

    pub fn map_size(&self) -> Size2<MInt> {
        self.map_size
    }

    fn get_unit<'a>(&'a self, id: UnitId) -> &'a Unit {
        match self.game_state.units.find(&id) {
            Some(unit) => unit,
            None => fail!("!"),
        }
    }

    fn hit_test(&self, attacker_id: UnitId, defender_id: UnitId) -> bool {
        let attacker_type_id = self.get_unit(attacker_id).type_id;
        let defender_type_id = self.get_unit(defender_id).type_id;
        let needed = match (attacker_type_id, defender_type_id) { // TODO: rename
            (Tank, Tank) => 5,
            (Tank, Soldier) => 3,
            (Soldier, Tank) => 7,
            (Soldier, Soldier) => 5,
        };
        task_rng().gen_range(0, 10 + 1) > needed
    }

    pub fn player_id(&self) -> PlayerId {
        self.current_player_id
    }

    pub fn get_event(&mut self) -> Option<Event> {
        let list = self.event_lists.get_mut(&self.current_player_id);
        list.shift()
    }

    fn command_to_core_event(&self, command: Command) -> Box<CoreEvent> {
        match command {
            CommandEndTurn => {
                CoreEventEndTurn::new(self) as Box<CoreEvent>
            },
            CommandCreateUnit(pos) => {
                CoreEventCreateUnit::new(
                    self,
                    pos,
                    Tank, // TODO: replace Tank with ...
                    self.current_player_id,
                ) as Box<CoreEvent>
            },
            CommandMove(unit_id, path) => {
                CoreEventMove::new(self, unit_id, path) as Box<CoreEvent>
            },
            CommandAttackUnit(attacker_id, defender_id) => {
                CoreEventAttackUnit::new(
                    self,
                    attacker_id,
                    defender_id,
                    self.hit_test(attacker_id, defender_id),
                ) as Box<CoreEvent>
            },
        }
    }

    pub fn do_command(&mut self, command: Command) {
        let core_event = self.command_to_core_event(command);
        self.do_core_event(core_event);
    }

    fn do_core_event(&mut self, core_event: Box<CoreEvent>) {
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
    fn new(_: &Core, unit_id: UnitId, path: Vec<MapPos>) -> Box<CoreEventMove> {
        box CoreEventMove {
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
        let unit = core.game_state.units.get_mut(&self.unit_id);
        unit.pos = *self.path.last().unwrap();
    }
}

struct CoreEventEndTurn {
    old_id: PlayerId,
    new_id: PlayerId,
}

impl CoreEventEndTurn {
    fn new(core: &Core) -> Box<CoreEventEndTurn> {
        let old_id = core.current_player_id.id;
        let max_id = core.players.len() as MInt;
        let new_id = if old_id + 1 == max_id { 0 } else { old_id + 1 };
        box CoreEventEndTurn {
            old_id: PlayerId{id: old_id},
            new_id: PlayerId{id: new_id},
        }
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
    type_id: UnitTypeId,
    player_id: PlayerId,
}

impl CoreEventCreateUnit {
    fn new(
        core: &Core,
        pos: MapPos,
        type_id: UnitTypeId,
        player_id: PlayerId
    ) -> Box<CoreEventCreateUnit> {
        let new_id = match core.game_state.units.keys().max_by(|&n| n) {
            Some(n) => n.id + 1,
            None => 0,
        };
        box CoreEventCreateUnit {
            id: UnitId{id: new_id},
            pos: pos,
            type_id: type_id,
            player_id: player_id,
        }
    }
}

impl CoreEvent for CoreEventCreateUnit {
    fn to_event(&self) -> Event {
        EventCreateUnit(self.id, self.pos, self.type_id, self.player_id)
    }

    fn apply(&self, core: &mut Core) {
        assert!(core.game_state.units.find(&self.id).is_none());
        let slot_id = match core.game_state.get_free_slot(self.id, self.pos) {
            Some(id) => id,
            None => fail!("No free slot in {}", self.pos),
        };
        core.game_state.units.insert(self.id, Unit {
            id: self.id,
            pos: self.pos,
            slot_id: slot_id,
            type_id: self.type_id,
            player_id: core.current_player_id,
        });
    }
}

struct CoreEventAttackUnit {
    attacker_id: UnitId,
    defender_id: UnitId,
    killed: bool,
}

impl CoreEventAttackUnit {
    fn new(
        _: &Core,
        attacker_id: UnitId,
        defender_id: UnitId,
        killed: bool
    ) -> Box<CoreEventAttackUnit> {
        println!("killed: {}", killed);
        box CoreEventAttackUnit {
            attacker_id: attacker_id,
            defender_id: defender_id,
            killed: killed,
        }
    }
}

impl CoreEvent for CoreEventAttackUnit {
    fn to_event(&self) -> Event {
        EventAttackUnit(self.attacker_id, self.defender_id, self.killed)
    }

    fn apply(&self, core: &mut Core) {
        if self.killed {
            assert!(core.game_state.units.find(&self.defender_id).is_some());
            core.game_state.units.remove(&self.defender_id);
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
