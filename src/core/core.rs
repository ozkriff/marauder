// See LICENSE file for copyright and license details.

use std::rand::{task_rng, Rng};
use std::collections::hashmap::HashMap;
use cgmath::vector::Vector2;
use error_context;
use core::types::{Size2, MInt, UnitId, PlayerId, MapPos};
use core::conf::Config;
use core::game_state::GameState;
use core::fs::FileSystem;
use core::map::{distance};

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

pub enum UnitClass {
    Infantry,
    Vehicle,
}

pub struct WeaponType {
    pub name: String,
    pub damage: MInt,
    pub ap: MInt,
    pub accuracy: MInt,
    pub max_distance: MInt,
}

#[deriving(Clone)]
pub struct WeaponTypeId{pub id: MInt}

pub struct UnitType {
    pub name: String,
    pub class: UnitClass,
    pub count: MInt,
    pub size: MInt,
    pub armor: MInt,
    pub toughness: MInt,
    pub weapon_skill: MInt,
    pub weapon_type_id: WeaponTypeId,
    pub move_points: MInt,
}

#[deriving(Clone)]
pub struct UnitTypeId{pub id: MInt}

pub struct Unit {
    pub id: UnitId,
    pub pos: MapPos,
    pub player_id: PlayerId,
    pub type_id: UnitTypeId,
    pub move_points: MInt,
    pub attacked: bool,
}

pub struct ObjectTypes {
    unit_types: Vec<UnitType>,
    weapon_types: Vec<WeaponType>,
}

impl ObjectTypes {
    pub fn new() -> ObjectTypes {
        let mut object_types = ObjectTypes {
            unit_types: vec![],
            weapon_types: vec![],
        };
        object_types.get_weapon_types();
        object_types.get_unit_types();
        object_types
    }

    // TODO: read from json/toml config
    fn get_weapon_types(&mut self) {
        self.weapon_types.push(WeaponType {
            name: "cannon".to_string(),
            damage: 9,
            ap: 9,
            accuracy: 5,
            max_distance: 5,
        });
        self.weapon_types.push(WeaponType {
            name: "rifle".to_string(),
            damage: 2,
            ap: 1,
            accuracy: 5,
            max_distance: 3,
        });
    }

    // TODO: read from json/toml config
    fn get_unit_types(&mut self) {
        let cannon_id = self.get_weapon_type_id("cannon");
        let rifle_id = self.get_weapon_type_id("rifle");
        self.unit_types.push(UnitType {
            name: "tank".to_string(),
            class: Vehicle,
            size: 6,
            count: 1,
            armor: 11,
            toughness: 9,
            weapon_skill: 5,
            weapon_type_id: cannon_id,
            move_points: 5,
        });
        self.unit_types.push(UnitType {
            name: "soldier".to_string(),
            class: Infantry,
            size: 4,
            count: 4,
            armor: 1,
            toughness: 2,
            weapon_skill: 5,
            weapon_type_id: rifle_id,
            move_points: 3,
        });
    }

    fn get_unit_type_id_opt(&self, name: &str) -> Option<UnitTypeId> {
        for (id, unit_type) in self.unit_types.iter().enumerate() {
            if unit_type.name.as_slice() == name {
                return Some(UnitTypeId{id: id as MInt});
            }
        }
        None
    }

    pub fn get_unit_type<'a>(&'a self, unit_type_id: UnitTypeId) -> &'a UnitType {
        &self.unit_types[unit_type_id.id as uint]
    }

    fn get_unit_type_id(&self, name: &str) -> UnitTypeId {
        match self.get_unit_type_id_opt(name) {
            Some(id) => id,
            None => fail!("No unit type with name: \"{}\"", name),
        }
    }

    fn get_weapon_type_id(&self, name: &str) -> WeaponTypeId {
        for (id, weapon_type) in self.weapon_types.iter().enumerate() {
            if weapon_type.name.as_slice() == name {
                return WeaponTypeId{id: id as MInt};
            }
        }
        fail!("No weapon type with name \"{}\"", name);
    }
}

pub struct Core {
    game_state: GameState,
    players: Vec<Player>,
    current_player_id: PlayerId,
    core_event_list: Vec<Event>,
    event_lists: HashMap<PlayerId, Vec<Event>>,
    map_size: Size2<MInt>,
    object_types: ObjectTypes,
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
            object_types: ObjectTypes::new(),
        };
        core.get_units();
        core
    }

    pub fn object_types(&self) -> &ObjectTypes {
        &self.object_types
    }

    // TODO: Move to scenario.json
    fn get_units(&mut self) {
        let tank_id = self.object_types.get_unit_type_id("tank");
        let soldier_id = self.object_types.get_unit_type_id("soldier");
        self.add_unit(MapPos{v: Vector2{x: 0, y: 0}}, tank_id, PlayerId{id: 0});
        self.add_unit(MapPos{v: Vector2{x: 0, y: 1}}, soldier_id, PlayerId{id: 0});
        self.add_unit(MapPos{v: Vector2{x: 2, y: 0}}, tank_id, PlayerId{id: 1});
        self.add_unit(MapPos{v: Vector2{x: 2, y: 2}}, soldier_id, PlayerId{id: 1});
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
            None => fail!("No unit with id = {}", id.id),
        }
    }

    pub fn get_weapon_type(&self, weapon_type_id: WeaponTypeId) -> &WeaponType {
        &self.object_types.weapon_types[weapon_type_id.id as uint]
    }

    fn hit_test(&self, attacker_id: UnitId, defender_id: UnitId) -> bool {
        fn test(needed: MInt) -> bool {
            let real = task_rng().gen_range(-5i32, 5i32);
            let result = real < needed;
            println!("real:{} < needed:{} = {}", real, needed, result);
            result
        }
        println!("");
        let attacker = self.get_unit(attacker_id);
        let defender = self.get_unit(defender_id);
        let attacker_type = self.object_types.get_unit_type(attacker.type_id);
        let defender_type = self.object_types.get_unit_type(defender.type_id);
        let weapon_type = self.get_weapon_type(attacker_type.weapon_type_id);
        if distance(attacker.pos, defender.pos) > weapon_type.max_distance {
            return false;
        }
        let hit_test_v = -15 + defender_type.size
            + weapon_type.accuracy + attacker_type.weapon_skill;
        let pierce_test_v = 5 + -defender_type.armor + weapon_type.ap;
        let wound_test_v = -defender_type.toughness + weapon_type.damage;
        println!("hit_test = {}, pierce_test = {}, wound_test_v = {}",
            hit_test_v, pierce_test_v, wound_test_v);
        print!("hit test: ");
        if !test(hit_test_v) {
            return false;
        }
        print!("pierce test: ");
        if !test(pierce_test_v) {
            return false;
        }
        print!("wound test: ");
        if !test(wound_test_v) {
            return false;
        }
        println!("HIT!");
        true
    }

    pub fn player_id(&self) -> PlayerId {
        self.current_player_id
    }

    pub fn get_event(&mut self) -> Option<Event> {
        let list = self.event_lists.get_mut(&self.current_player_id);
        list.remove(0)
    }

    fn command_attack_unit_to_event(
        &self,
        attacker_id: UnitId,
        defender_id: UnitId
    ) -> Option<Event> {
        let attacker = self.game_state.units.get(&attacker_id);
        let defender = self.game_state.units.get(&defender_id);
        let attacker_type = self.object_types.get_unit_type(attacker.type_id);
        let weapon_type = self.get_weapon_type(attacker_type.weapon_type_id);
        if distance(attacker.pos, defender.pos) <= weapon_type.max_distance {
            Some(EventAttackUnit(
                attacker_id,
                defender_id,
                self.hit_test(attacker_id, defender_id),
            ))
        } else {
            None
        }
    }

    fn command_to_event(&self, command: Command) -> Option<Event> {
        match command {
            CommandEndTurn => {
                let old_id = self.current_player_id.id;
                let max_id = self.players.len() as MInt;
                let new_id = if old_id + 1 == max_id {
                    0
                } else {
                    old_id + 1
                };
                Some(EventEndTurn(PlayerId{id: old_id}, PlayerId{id: new_id}))
            },
            CommandCreateUnit(pos) => {
                Some(EventCreateUnit(
                    self.get_new_unit_id(),
                    pos,
                    self.object_types.get_unit_type_id("soldier"),
                    self.current_player_id,
                ))
            },
            CommandMove(unit_id, path) => {
                Some(EventMove(unit_id, path))
            },
            CommandAttackUnit(attacker_id, defender_id) => {
                self.command_attack_unit_to_event(attacker_id, defender_id)
            },
        }
    }

    pub fn do_command(&mut self, command: Command) {
        match self.command_to_event(command) {
            Some(event) => self.do_core_event(event),
            None => {},
        }
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
            self.game_state.apply_event(&self.object_types, &event);
            for player in self.players.iter() {
                let event_list = self.event_lists.get_mut(&player.id);
                // TODO: per player event filter
                event_list.push(event.clone());
            }
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
