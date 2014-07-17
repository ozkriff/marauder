// See LICENSE file for copyright and license details.

use std::rand;
use std::rand::Rng;
use cgmath::vector::{Vector3, Vector, EuclideanVector};
use visualizer::geom;
use core::types::{MapPos, UnitId};
use core::game_state::GameState;
use core::core;
use visualizer::mesh::{MeshId};
use visualizer::scene::{
    Scene,
    SceneNode,
    NodeId,
    MAX_UNIT_NODE_ID,
    SHELL_NODE_ID,
};
use visualizer::types::{MFloat, WorldPos, Time};

fn unit_id_to_node_id(unit_id: UnitId) -> NodeId {
    NodeId{id: unit_id.id}
}

fn marker_id(unit_id: UnitId) -> NodeId {
    NodeId{id: MAX_UNIT_NODE_ID.id + unit_id.id}
}

pub trait EventVisualizer {
    fn is_finished(&self) -> bool;
    fn draw(&mut self, scene: &mut Scene, dtime: Time);
    fn end(&mut self, scene: &mut Scene, state: &GameState);
}

pub struct EventMoveVisualizer {
    unit_id: UnitId,
    path: Vec<WorldPos>,
    move: MoveHelper,
    speed: MFloat,
}

impl EventVisualizer for EventMoveVisualizer {
    fn is_finished(&self) -> bool {
        self.path.len() == 1
    }

    fn draw(&mut self, scene: &mut Scene, dtime: Time) {
        let pos = self.move.step(dtime);
        {
            let marker_node = scene.nodes.get_mut(&marker_id(self.unit_id));
            marker_node.pos.v = pos.v.add_v(&vec3_z(geom::HEX_EX_RADIUS / 2.0));
        }
        let node_id = unit_id_to_node_id(self.unit_id);
        let node = scene.nodes.get_mut(&node_id);
        node.pos = pos;
        if self.move.is_finished() {
            self.path.shift();
            if self.path.len() > 1 {
                self.update_waypoint(node);
            }
            node.pos = self.current_waypoint();
        }
    }

    fn end(&mut self, scene: &mut Scene, _: &GameState) {
        assert!(self.path.len() == 1);
        let node_id = unit_id_to_node_id(self.unit_id);
        let node = scene.nodes.get_mut(&node_id);
        node.pos = self.current_waypoint();
    }
}

impl EventMoveVisualizer {
    pub fn new(
        scene: &mut Scene,
        _: &GameState,
        unit_id: UnitId,
        path: Vec<MapPos>
    ) -> Box<EventVisualizer> {
        let mut world_path = Vec::new();
        for map_pos in path.iter() {
            let world_pos = geom::map_pos_to_world_pos(*map_pos);
            world_path.push(world_pos);
        }
        let speed = 3.8; // TODO: Get from UnitType
        let node_id = unit_id_to_node_id(unit_id);
        let node = scene.nodes.get_mut(&node_id);
        node.rot = geom::get_rot_angle(
            *world_path.get(0), *world_path.get(1));
        let move = MoveHelper::new(
            *world_path.get(0), *world_path.get(1), speed);
        let mut vis = box EventMoveVisualizer {
            unit_id: unit_id,
            path: world_path,
            move: move,
            speed: speed,
        };
        vis.update_waypoint(node);
        vis as Box<EventVisualizer>
    }

    fn update_waypoint(&mut self, node: &mut SceneNode) {
        self.move = MoveHelper::new(
            self.current_waypoint(),
            self.next_waypoint(),
            self.speed,
        );
        node.rot = geom::get_rot_angle(
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
    pub fn new() -> Box<EventVisualizer> {
        box EventEndTurnVisualizer as Box<EventVisualizer>
    }
}

impl EventVisualizer for EventEndTurnVisualizer {
    fn is_finished(&self) -> bool {
        true
    }

    fn draw(&mut self, _: &mut Scene, _: Time) {}

    fn end(&mut self, _: &mut Scene, _: &GameState) {}
}

pub struct EventCreateUnitVisualizer {
    id: UnitId,
    move: MoveHelper,
}

fn get_unit_scene_nodes(
    type_id: core::UnitTypeId,
    mesh_id: MeshId,
) -> Vec<SceneNode> {
    match type_id {
        core::Tank => vec![
            SceneNode {
                pos: WorldPos{v: Vector3{x: 0.0, y: 0.0, z: 0.0}},
                rot: 0.0,
                mesh_id: Some(mesh_id),
                children: vec![],
            },
        ],
        core::Soldier => vec![
            SceneNode {
                pos: WorldPos{v: Vector3{x: 0.2, y: 0.2, z: 0.0}},
                rot: 0.0,
                mesh_id: Some(mesh_id),
                children: vec![],
            },
            SceneNode {
                pos: WorldPos{v: Vector3{x: 0.2, y: -0.2, z: 0.0}},
                rot: 0.0,
                mesh_id: Some(mesh_id),
                children: vec![],
            },
            SceneNode {
                pos: WorldPos{v: Vector3{x: -0.2, y: 0.2, z: 0.0}},
                rot: 0.0,
                mesh_id: Some(mesh_id),
                children: vec![],
            },
            SceneNode {
                pos: WorldPos{v: Vector3{x: -0.2, y: -0.2, z: 0.0}},
                rot: 0.0,
                mesh_id: Some(mesh_id),
                children: vec![],
            },
        ],
    }
}

impl EventCreateUnitVisualizer {
    pub fn new(
        scene: &mut Scene,
        _: &GameState,
        id: UnitId,
        type_id: core::UnitTypeId,
        pos: MapPos,
        mesh_id: MeshId,
        marker_mesh_id: MeshId
    ) -> Box<EventVisualizer> {
        let node_id = unit_id_to_node_id(id);
        let world_pos = geom::map_pos_to_world_pos(pos);
        let to = world_pos;
        let from = WorldPos{v: to.v.sub_v(&vec3_z(geom::HEX_EX_RADIUS / 2.0))};
        let rot = rand::task_rng().gen_range::<MFloat>(0.0, 360.0);
        scene.nodes.insert(node_id, SceneNode {
            pos: from,
            rot: rot,
            mesh_id: None,
            children: get_unit_scene_nodes(type_id, mesh_id),
        });
        scene.nodes.insert(marker_id(id), SceneNode {
            pos: WorldPos{v: to.v.add_v(&vec3_z(geom::HEX_EX_RADIUS / 2.0))},
            rot: 0.0,
            mesh_id: Some(marker_mesh_id),
            children: Vec::new(),
        });
        let move = MoveHelper::new(from, to, 1.0);
        box EventCreateUnitVisualizer {
            id: id,
            move: move,
        } as Box<EventVisualizer>
    }
}

impl EventVisualizer for EventCreateUnitVisualizer {
    fn is_finished(&self) -> bool {
        self.move.is_finished()
    }

    fn draw(&mut self, scene: &mut Scene, dtime: Time) {
        let node_id = unit_id_to_node_id(self.id);
        let node = scene.nodes.get_mut(&node_id);
        node.pos = self.move.step(dtime);
    }

    fn end(&mut self, _: &mut Scene, _: &GameState) {}
}

pub struct MoveHelper {
    to: WorldPos,
    current: WorldPos,
    dist: MFloat,
    current_dist: MFloat,
    dir: Vector3<MFloat>,
}

impl MoveHelper {
    pub fn new(
        from: WorldPos,
        to: WorldPos,
        speed: MFloat
    ) -> MoveHelper {
        let dir = to.v.sub_v(&from.v).normalize();
        let dist = geom::dist(from, to);
        MoveHelper {
            to: to,
            current: from,
            dist: dist,
            current_dist: 0.0,
            dir: dir.mul_s(speed),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.current_dist >= self.dist
    }

    pub fn step(&mut self, dtime: Time) -> WorldPos {
        let dt = dtime.n as MFloat / 1000000000.0;
        let step = self.dir.mul_s(dt);
        self.current_dist += step.length();
        self.current.v.add_self_v(&step);
        if self.is_finished() {
            self.current = self.to;
        }
        self.current
    }
}

fn vec3_z(z: MFloat) -> Vector3<MFloat> {
    Vector3{x: 0.0, y: 0.0, z: z}
}

pub struct EventAttackUnitVisualizer {
    defender_id: UnitId,
    killed: bool,
    move: MoveHelper,
    shell_move: MoveHelper,
}

impl EventAttackUnitVisualizer {
    pub fn new(
        scene: &mut Scene,
        _: &GameState,
        attacker_id: UnitId,
        defender_id: UnitId,
        killed: bool,
        shell_mesh_id: MeshId
    ) -> Box<EventVisualizer> {
        let node_id = unit_id_to_node_id(defender_id);
        let from = scene.nodes.get(&node_id).pos;
        let to = WorldPos{v: from.v.sub_v(&vec3_z(geom::HEX_EX_RADIUS / 2.0))};
        let move = MoveHelper::new(from, to, 1.0);
        let shell_move = {
            let from = scene.nodes.get(&unit_id_to_node_id(attacker_id)).pos;
            let to = scene.nodes.get(&unit_id_to_node_id(defender_id)).pos;
            scene.nodes.insert(SHELL_NODE_ID, SceneNode {
                pos: from,
                rot: 0.0,
                mesh_id: Some(shell_mesh_id),
                children: Vec::new(),
            });
            MoveHelper::new(from, to, 10.0)
        };
        box EventAttackUnitVisualizer {
            defender_id: defender_id,
            killed: killed,
            move: move,
            shell_move: shell_move,
        } as Box<EventVisualizer>
    }
}

impl EventVisualizer for EventAttackUnitVisualizer {
    fn is_finished(&self) -> bool {
        if self.killed {
            self.move.is_finished()
        } else {
            self.shell_move.is_finished()
        }
    }

    fn draw(&mut self, scene: &mut Scene, dtime: Time) {
        scene.nodes.get_mut(&SHELL_NODE_ID).pos = self.shell_move.step(dtime);
        if self.killed {
            if self.shell_move.is_finished() {
                let node_id = unit_id_to_node_id(self.defender_id);
                scene.nodes.get_mut(&node_id).pos = self.move.step(dtime);
            }
        }
    }

    fn end(&mut self, scene: &mut Scene, _: &GameState) {
        if self.killed {
            let node_id = unit_id_to_node_id(self.defender_id);
            scene.nodes.remove(&node_id);
            scene.nodes.remove(&marker_id(self.defender_id));
        }
        scene.nodes.remove(&SHELL_NODE_ID);
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
