// See LICENSE file for copyright and license details.

use cgmath::vector::{Vec3, Vector, EuclideanVector};
use visualizer::geom::Geom;
use core::types::{MBool, MInt, MapPos, UnitId};
use core::game_state::GameState;
use visualizer::types::{Scene, SceneNode, MFloat, WorldPos, NodeId};

fn unit_id_to_node_id(unit_id: UnitId) -> NodeId {
    let UnitId(id) = unit_id;
    NodeId(id)
}

pub trait EventVisualizer {
    fn is_finished(&self) -> MBool;
    fn start(&mut self, geom: &Geom, scene: &mut Scene, state: &GameState);
    fn draw(&mut self, geom: &Geom, scene: &mut Scene, state: &GameState, dtime: MInt);
    fn end(&mut self, geom: &Geom, scene: &mut Scene, state: &GameState);
}

static MOVE_SPEED: MFloat = 40.0; // TODO: config?

// TODO: Replace with slot system
fn unit_pos(
    unit_id: UnitId,
    map_pos: MapPos,
    geom: &Geom,
    state: &GameState,
) -> WorldPos {
    let slot_id = state.get_slot_index(unit_id, map_pos);
    let center_pos = geom.map_pos_to_world_pos(map_pos);
    let slot_pos = geom.slot_pos(slot_id);
    center_pos.add_v(&slot_pos)
}

pub struct EventMoveVisualizer {
    unit_id: UnitId,
    path: ~[WorldPos],
    dist: MFloat,
    current_dist: MFloat,
    dir: Vec3<MFloat>,
}

impl EventVisualizer for EventMoveVisualizer {
    fn start(&mut self, geom: &Geom, scene: &mut Scene, _: &GameState) {
        self.reset_dir(geom);
        let node_id = unit_id_to_node_id(self.unit_id);
        let node = scene.get_mut(&node_id);
        node.rot = geom.get_rot_angle(
            self.current_waypoint(), self.next_waypoint());
    }

    fn is_finished(&self) -> MBool {
        self.path.len() == 1
    }

    fn draw(&mut self, geom: &Geom, scene: &mut Scene, _: &GameState, dtime: MInt) {
        let node_id = unit_id_to_node_id(self.unit_id);
        let node = scene.get_mut(&node_id);
        let dt = dtime as MFloat / 1000000000.0;
        let speed = 3.8; // TODO: Get from UnitType
        let step = self.dir.mul_s(dt).mul_s(speed);
        node.pos.add_self_v(&step);
        self.current_dist += step.length();
        if self.current_dist > self.dist {
            let _ = self.path.shift();
            if self.path.len() > 1 {
                self.reset_dir(geom);
                node.rot = geom.get_rot_angle(
                    self.current_waypoint(), self.next_waypoint());
            }
            node.pos = self.current_waypoint();
        }
    }

    fn end(&mut self, _: &Geom, scene: &mut Scene, _: &GameState) {
        assert!(self.path.len() == 1);
        let node_id = unit_id_to_node_id(self.unit_id);
        let unit_node = scene.get_mut(&node_id);
        unit_node.pos = self.current_waypoint();
    }
}

impl EventMoveVisualizer {
    // TODO: Merge 'new' and 'start'
    pub fn new(
        geom: &Geom,
        _: &mut Scene,
        state: &GameState,
        unit_id: UnitId,
        path: ~[MapPos]
    ) -> ~EventVisualizer {
        let mut world_path = ~[];
        for map_pos in path.iter() {
            let world_pos = unit_pos(unit_id, *map_pos, geom, state);
            world_path.push(world_pos);
        }
        ~EventMoveVisualizer {
            unit_id: unit_id,
            path: world_path,
            dist: 0.0,
            current_dist: 0.0,
            dir: Vec3::zero(),
        } as ~EventVisualizer
    }

    fn reset_dir(&mut self, geom: &Geom) {
        let next = self.next_waypoint();
        let current = self.current_waypoint();
        self.dist = geom.dist(current, next);
        self.dir = next.sub_v(&current).normalize();
        self.current_dist = 0.0;
    }

    fn current_waypoint(&self) -> WorldPos {
        assert!(self.path.len() >= 1);
        self.path[0]
    }

    fn next_waypoint(&self) -> WorldPos {
        assert!(self.path.len() >= 2);
        self.path[1]
    }
}

pub struct EventEndTurnVisualizer;

impl EventEndTurnVisualizer {
    pub fn new() -> ~EventVisualizer {
        ~EventEndTurnVisualizer as ~EventVisualizer
    }
}

impl EventVisualizer for EventEndTurnVisualizer {
    fn start(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}

    fn is_finished(&self) -> MBool {
        true
    }

    fn draw(&mut self, _: &Geom, _: &mut Scene, _: &GameState, _: MInt) {}

    fn end(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}
}

pub struct EventCreateUnitVisualizer {
    id: UnitId,
    pos: MapPos,
    anim_index: MInt,
}

impl EventCreateUnitVisualizer {
    pub fn new(id: UnitId, pos: MapPos) -> ~EventVisualizer {
        ~EventCreateUnitVisualizer {
            id: id,
            pos: pos,
            anim_index: 0,
        } as ~EventVisualizer
    }
}

impl EventVisualizer for EventCreateUnitVisualizer {
    fn start(&mut self, geom: &Geom, scene: &mut Scene, state: &GameState) {
        let node_id = unit_id_to_node_id(self.id);
        let world_pos = unit_pos(self.id, self.pos, geom, state);
        scene.insert(node_id, SceneNode{pos: world_pos, rot: 0.0});
    }

    fn is_finished(&self) -> MBool {
        self.anim_index == MOVE_SPEED as MInt
    }

    fn draw(&mut self, geom: &Geom, scene: &mut Scene, state: &GameState, _: MInt) {
        let node_id = unit_id_to_node_id(self.id);
        let mut pos = unit_pos(self.id, self.pos, geom, state);
        pos.z -= 0.02 * (MOVE_SPEED / self.anim_index as MFloat);
        scene.get_mut(&node_id).pos = pos;
        self.anim_index += 1;
    }

    fn end(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}
}

pub struct EventAttackUnitVisualizer {
    attacker_id: UnitId,
    defender_id: UnitId,
    anim_index: MInt,
}

impl EventAttackUnitVisualizer {
    pub fn new(attacker_id: UnitId, defender_id: UnitId) -> ~EventVisualizer {
        ~EventAttackUnitVisualizer {
            attacker_id: attacker_id,
            defender_id: defender_id,
            anim_index: 0,
        } as ~EventVisualizer
    }
}

impl EventVisualizer for EventAttackUnitVisualizer {
    fn start(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}

    fn is_finished(&self) -> MBool {
        self.anim_index == MOVE_SPEED as MInt
    }

    fn draw(&mut self, _: &Geom, scene: &mut Scene, _: &GameState, _: MInt) {
        let node_id = unit_id_to_node_id(self.defender_id);
        scene.get_mut(&node_id).pos.z -= 0.01;
        self.anim_index += 1;
    }

    fn end(&mut self, _: &Geom, scene: &mut Scene, _: &GameState) {
        let node_id = unit_id_to_node_id(self.defender_id);
        scene.remove(&node_id);
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
