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
    move: MoveHelper,
    speed: MFloat,
}

impl EventVisualizer for EventMoveVisualizer {
    fn start(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}

    fn is_finished(&self) -> MBool {
        self.path.len() == 1
    }

    fn draw(&mut self, geom: &Geom, scene: &mut Scene, _: &GameState, dtime: MInt) {
        let node_id = unit_id_to_node_id(self.unit_id);
        let node = scene.get_mut(&node_id);
        node.pos = self.move.step(dtime);
        if self.move.is_finished() {
            self.path.shift();
            if self.path.len() > 1 {
                self.update_waypoint(geom, node);
            }
            node.pos = self.current_waypoint();
        }
    }

    fn end(&mut self, _: &Geom, scene: &mut Scene, _: &GameState) {
        assert!(self.path.len() == 1);
        let node_id = unit_id_to_node_id(self.unit_id);
        let node = scene.get_mut(&node_id);
        node.pos = self.current_waypoint();
    }
}

impl EventMoveVisualizer {
    // TODO: Merge 'new' and 'start'
    pub fn new(
        geom: &Geom,
        scene: &mut Scene,
        state: &GameState,
        unit_id: UnitId,
        path: ~[MapPos]
    ) -> ~EventVisualizer {
        let mut world_path = ~[];
        for map_pos in path.iter() {
            let world_pos = unit_pos(unit_id, *map_pos, geom, state);
            world_path.push(world_pos);
        }
        let speed = 3.8; // TODO: Get from UnitType
        let node_id = unit_id_to_node_id(unit_id);
        let node = scene.get_mut(&node_id);
        node.rot = geom.get_rot_angle(world_path[0], world_path[1]);
        let move = MoveHelper::new(geom, world_path[0], world_path[1], speed);
        let mut vis = ~EventMoveVisualizer {
            unit_id: unit_id,
            path: world_path,
            move: move,
            speed: speed,
        };
        vis.update_waypoint(geom, node);
        vis as ~EventVisualizer
    }

    fn update_waypoint(&mut self, geom: &Geom, node: &mut SceneNode) {
        self.move = MoveHelper::new(
            geom,
            self.current_waypoint(),
            self.next_waypoint(),
            self.speed,
        );
        node.rot = geom.get_rot_angle(
            self.current_waypoint(),
            self.next_waypoint()
        );
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
    move: MoveHelper,
}

impl EventCreateUnitVisualizer {
    pub fn new(
        geom: &Geom,
        scene: &mut Scene,
        state: &GameState,
        id: UnitId,
        pos: MapPos
    ) -> ~EventVisualizer {
        let node_id = unit_id_to_node_id(id);
        let world_pos = unit_pos(id, pos, geom, state);
        let to = world_pos;
        let from = to.sub_v(&vec3_z(geom.hex_ex_radius / 2.0));
        scene.insert(node_id, SceneNode{pos: from, rot: 0.0});
        let move = MoveHelper::new(geom, from, to, 1.0);
        ~EventCreateUnitVisualizer {
            id: id,
            move: move,
        } as ~EventVisualizer
    }
}

impl EventVisualizer for EventCreateUnitVisualizer {
    fn start(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}

    fn is_finished(&self) -> MBool {
        self.move.is_finished()
    }

    fn draw(&mut self, _: &Geom, scene: &mut Scene, _: &GameState, dtime: MInt) {
        let node_id = unit_id_to_node_id(self.id);
        let node = scene.get_mut(&node_id);
        node.pos = self.move.step(dtime);
    }

    fn end(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}
}

pub struct MoveHelper {
    from: WorldPos,
    to: WorldPos,
    current: WorldPos,
    dist: MFloat,
    current_dist: MFloat,
    dir: Vec3<MFloat>,
}

impl MoveHelper {
    pub fn new(
        geom: &Geom,
        from: WorldPos,
        to: WorldPos,
        speed: MFloat
    ) -> MoveHelper {
        let dir = to.sub_v(&from).normalize();
        let dist = geom.dist(from, to);
        MoveHelper {
            from: from,
            to: to,
            current: from,
            dist: dist,
            current_dist: 0.0,
            dir: dir.mul_s(speed),
        }
    }

    pub fn is_finished(&self) -> MBool {
        self.current_dist >= self.dist
    }

    pub fn step(&mut self, dtime: MInt) -> WorldPos {
        let dt = dtime as MFloat / 1000000000.0;
        let step = self.dir.mul_s(dt);
        self.current_dist += step.length();
        self.current.add_self_v(&step);
        self.current
    }
}

fn vec3_z(z: MFloat) -> Vec3<MFloat> {
    Vec3{x: 0.0, y: 0.0, z: z}
}

pub struct EventAttackUnitVisualizer {
    attacker_id: UnitId,
    defender_id: UnitId,
    move: MoveHelper,
}

impl EventAttackUnitVisualizer {
    pub fn new(
        geom: &Geom,
        scene: &mut Scene,
        _: &GameState,
        attacker_id: UnitId,
        defender_id: UnitId
    ) -> ~EventVisualizer {
        let node_id = unit_id_to_node_id(defender_id);
        let node = scene.get_mut(&node_id);
        let from = node.pos;
        let to = from.sub_v(&vec3_z(geom.hex_ex_radius / 2.0));
        let move = MoveHelper::new(geom, from, to, 1.0);
        ~EventAttackUnitVisualizer {
            attacker_id: attacker_id,
            defender_id: defender_id,
            move: move,
        } as ~EventVisualizer
    }
}

impl EventVisualizer for EventAttackUnitVisualizer {
    fn start(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}

    fn is_finished(&self) -> MBool {
        self.move.is_finished()
    }

    fn draw(&mut self, _: &Geom, scene: &mut Scene, _: &GameState, dtime: MInt) {
        let node_id = unit_id_to_node_id(self.defender_id);
        let node = scene.get_mut(&node_id);
        node.pos = self.move.step(dtime);
    }

    fn end(&mut self, _: &Geom, scene: &mut Scene, _: &GameState) {
        let node_id = unit_id_to_node_id(self.defender_id);
        scene.remove(&node_id);
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
