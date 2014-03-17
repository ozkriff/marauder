// See LICENSE file for copyright and license details.

use std::vec_ng::Vec;
use rand;
use rand::Rng;
use cgmath::vector::{Vec3, Vector, EuclideanVector};
use visualizer::geom::Geom;
use core::types::{MBool, MInt, MapPos, UnitId};
use core::game_state::GameState;
use visualizer::types::{Scene, SceneNode, MFloat, WorldPos, NodeId};

fn unit_id_to_node_id(unit_id: UnitId) -> NodeId {
    let UnitId(id) = unit_id;
    NodeId(id)
}

fn marker_id(unit_id: UnitId) -> NodeId {
    let UnitId(id) = unit_id;
    NodeId(id + 1000)
}

pub trait EventVisualizer {
    fn is_finished(&self) -> MBool;
    fn draw(&mut self, geom: &Geom, scene: &mut Scene, dtime: MInt);
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
    priv unit_id: UnitId,
    priv path: Vec<WorldPos>,
    priv move: MoveHelper,
    priv speed: MFloat,
}

impl EventVisualizer for EventMoveVisualizer {
    fn is_finished(&self) -> MBool {
        self.path.len() == 1
    }

    fn draw(&mut self, geom: &Geom, scene: &mut Scene, dtime: MInt) {
        let pos = self.move.step(dtime);
        {
            let marker_node = scene.get_mut(&marker_id(self.unit_id));
            marker_node.pos = pos.add_v(&vec3_z(geom.hex_ex_radius / 2.0));
        }
        let node_id = unit_id_to_node_id(self.unit_id);
        let node = scene.get_mut(&node_id);
        node.pos = pos;
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
    pub fn new(
        geom: &Geom,
        scene: &mut Scene,
        state: &GameState,
        unit_id: UnitId,
        path: Vec<MapPos>
    ) -> ~EventVisualizer {
        let mut world_path = Vec::new();
        for map_pos in path.iter() {
            let world_pos = unit_pos(unit_id, *map_pos, geom, state);
            world_path.push(world_pos);
        }
        let speed = 3.8; // TODO: Get from UnitType
        let node_id = unit_id_to_node_id(unit_id);
        let node = scene.get_mut(&node_id);
        node.rot = geom.get_rot_angle(*world_path.get(0), *world_path.get(1));
        let move = MoveHelper::new(geom, *world_path.get(0), *world_path.get(1), speed);
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
        *self.path.get(0)
    }

    fn next_waypoint(&self) -> WorldPos {
        assert!(self.path.len() >= 2);
        *self.path.get(1)
    }
}

pub struct EventEndTurnVisualizer;

impl EventEndTurnVisualizer {
    pub fn new() -> ~EventVisualizer {
        ~EventEndTurnVisualizer as ~EventVisualizer
    }
}

impl EventVisualizer for EventEndTurnVisualizer {
    fn is_finished(&self) -> MBool {
        true
    }

    fn draw(&mut self, _: &Geom, _: &mut Scene, _: MInt) {}

    fn end(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}
}

pub struct EventCreateUnitVisualizer {
    priv id: UnitId,
    priv move: MoveHelper,
}

impl EventCreateUnitVisualizer {
    pub fn new(
        geom: &Geom,
        scene: &mut Scene,
        state: &GameState,
        id: UnitId,
        pos: MapPos,
        mesh_id: MInt,
        marker_mesh_id: MInt
    ) -> ~EventVisualizer {
        let node_id = unit_id_to_node_id(id);
        let world_pos = unit_pos(id, pos, geom, state);
        let to = world_pos;
        let from = to.sub_v(&vec3_z(geom.hex_ex_radius / 2.0));
        let rot = rand::task_rng().gen_range::<MFloat>(0.0, 360.0);
        scene.insert(node_id, SceneNode {
            pos: from,
            rot: rot,
            mesh_id: mesh_id,
        });
        scene.insert(marker_id(id), SceneNode {
            pos: to.add_v(&vec3_z(geom.hex_ex_radius / 2.0)),
            rot: 0.0,
            mesh_id: marker_mesh_id,
        });
        let move = MoveHelper::new(geom, from, to, 1.0);
        ~EventCreateUnitVisualizer {
            id: id,
            move: move,
        } as ~EventVisualizer
    }
}

impl EventVisualizer for EventCreateUnitVisualizer {
    fn is_finished(&self) -> MBool {
        self.move.is_finished()
    }

    fn draw(&mut self, _: &Geom, scene: &mut Scene, dtime: MInt) {
        let node_id = unit_id_to_node_id(self.id);
        let node = scene.get_mut(&node_id);
        node.pos = self.move.step(dtime);
    }

    fn end(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}
}

pub struct MoveHelper {
    priv from: WorldPos,
    priv to: WorldPos,
    priv current: WorldPos,
    priv dist: MFloat,
    priv current_dist: MFloat,
    priv dir: Vec3<MFloat>,
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
    priv attacker_id: UnitId,
    priv defender_id: UnitId,
    priv move: MoveHelper,
    priv shell_move: MoveHelper,
    priv shell_node_id: NodeId,
}

impl EventAttackUnitVisualizer {
    pub fn new(
        geom: &Geom,
        scene: &mut Scene,
        _: &GameState,
        attacker_id: UnitId,
        defender_id: UnitId,
        shell_mesh_id: MInt
    ) -> ~EventVisualizer {
        let node_id = unit_id_to_node_id(defender_id);
        let from = scene.get(&node_id).pos;
        let to = from.sub_v(&vec3_z(geom.hex_ex_radius / 2.0));
        let move = MoveHelper::new(geom, from, to, 1.0);
        let shell_node_id = NodeId(666); // TODO
        let shell_move = {
            let from = scene.get(&unit_id_to_node_id(attacker_id)).pos;
            let to = scene.get(&unit_id_to_node_id(defender_id)).pos;
            scene.insert(shell_node_id, SceneNode {
                pos: from,
                rot: 0.0,
                mesh_id: shell_mesh_id,
            });
            MoveHelper::new(geom, from, to, 10.0)
        };
        ~EventAttackUnitVisualizer {
            attacker_id: attacker_id,
            defender_id: defender_id,
            move: move,
            shell_move: shell_move,
            shell_node_id: shell_node_id,
        } as ~EventVisualizer
    }
}

impl EventVisualizer for EventAttackUnitVisualizer {
    fn is_finished(&self) -> MBool {
        self.move.is_finished() && self.shell_move.is_finished()
    }

    fn draw(&mut self, _: &Geom, scene: &mut Scene, dtime: MInt) {
        scene.get_mut(&self.shell_node_id).pos = self.shell_move.step(dtime);
        if self.shell_move.is_finished() {
            let node_id = unit_id_to_node_id(self.defender_id);
            scene.get_mut(&node_id).pos = self.move.step(dtime);
        }
    }

    fn end(&mut self, _: &Geom, scene: &mut Scene, _: &GameState) {
        let node_id = unit_id_to_node_id(self.defender_id);
        scene.remove(&node_id);
        scene.remove(&self.shell_node_id);
        scene.remove(&marker_id(self.defender_id));
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
