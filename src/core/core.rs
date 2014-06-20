// See LICENSE file for copyright and license details.

use std::rand::{task_rng, Rng};
use std::collections::hashmap::HashMap;
use cgmath::vector::Vector2;
use error_context;
use core::types::{Size2, MInt, UnitId, PlayerId, MapPos};
use core::conf::Config;
use core::game_state::GameState;
use core::fs::FileSystem;

pub enum Command {
    CommandMove(UnitId, Vec<MapPos>),
    CommandEndTurn,
    CommandCreateUnit(MapPos),
    CommandAttackUnit(UnitId, UnitId),
}

#[deriving(Clone)]
pub enum Event {
    EventMove(UnitId, Vec<MapPos>),
    EventEndTurn(PlayerId, PlayerId), // old_id, new_id
    EventCreateUnit(UnitId, MapPos, UnitTypeId, PlayerId),
    EventAttackUnit(UnitId, UnitId, /* killed: */ bool),
}

pub struct Player {
    pub id: PlayerId,
}

#[deriving(Clone)]
pub enum UnitTypeId {
    Tank,
    Soldier,
}

pub struct Unit {
    pub id: UnitId,
    pub pos: MapPos,
    pub player_id: PlayerId,
    pub type_id: UnitTypeId,
}

pub struct Core {
    game_state: GameState,
    players: Vec<Player>,
    current_player_id: PlayerId,
    core_event_list: Vec<Event>,
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

    fn get_new_unit_id(&self) -> UnitId {
        let id = match self.game_state.units.keys().max_by(|&n| n) {
            Some(n) => n.id + 1,
            None => 0,
        };
        UnitId{id: id}
    }

    fn add_unit(&mut self, pos: MapPos, type_id: UnitTypeId, player_id: PlayerId) {
        let event = EventCreateUnit(
            self.get_new_unit_id(),
            pos,
            type_id,
            player_id,
        );
        self.do_core_event(event);
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

    fn command_to_event(&self, command: Command) -> Event {
        match command {
            CommandEndTurn => {
                let old_id = self.current_player_id.id;
                let max_id = self.players.len() as MInt;
                let new_id = if old_id + 1 == max_id {
                    0
                } else {
                    old_id + 1
                };
                EventEndTurn(PlayerId{id: old_id}, PlayerId{id: new_id})
            },
            CommandCreateUnit(pos) => {
                EventCreateUnit(
                    self.get_new_unit_id(),
                    pos,
                    Tank, // TODO: replace Tank with ...
                    self.current_player_id,
                )
            },
            CommandMove(unit_id, path) => {
                EventMove(unit_id, path)
            },
            CommandAttackUnit(attacker_id, defender_id) => {
                EventAttackUnit(
                    attacker_id,
                    defender_id,
                    self.hit_test(attacker_id, defender_id),
                )
            },
        }
    }

    pub fn do_command(&mut self, command: Command) {
        let event = self.command_to_event(command);
        self.do_core_event(event);
    }

    fn do_core_event(&mut self, core_event: Event) {
        self.core_event_list.push(core_event);
        self.make_events();
    }

    fn apply_event(&mut self, event: &Event) {
        match *event {
            EventEndTurn(old_player_id, new_player_id) => {
                for player in self.players.iter() {
                    if player.id == new_player_id {
                        if self.current_player_id == old_player_id {
                            self.current_player_id = player.id;
                        }
                        return;
                    }
                }
            },
            _ => {},
        }
    }

    fn make_events(&mut self) {
        while self.core_event_list.len() != 0 {
            let event = self.core_event_list.pop().unwrap();
            self.apply_event(&event);
            self.game_state.apply_event(&event);
            for player in self.players.iter() {
                let event_list = self.event_lists.get_mut(&player.id);
                // TODO: per player event filter
                event_list.push(event.clone());
            }
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
