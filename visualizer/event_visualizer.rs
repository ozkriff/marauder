// See LICENSE file for copyright and license details.

use cgmath::vector::Vector;
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
    fn draw(&mut self, geom: &Geom, scene: &mut Scene, state: &GameState);
    fn end(&mut self, geom: &Geom, scene: &mut Scene, state: &GameState);
}

static MOVE_SPEED: MFloat = 40.0; // TODO: config?

// TODO: Replace with slot system
fn unit_pos(
    unit_id: UnitId,
    map_pos: MapPos,
    geom: &Geom,
    state: &GameState
) -> WorldPos {
    let slot_id = state.get_slot_index(unit_id, map_pos);
    let center_pos = geom.map_pos_to_world_pos(map_pos);
    let slot_pos = geom.slot_pos(slot_id);
    center_pos.add_v(&slot_pos)
}

pub struct EventMoveVisualizer {
    unit_id: UnitId,
    path: ~[MapPos],
    current_move_index: MInt,
}

impl EventVisualizer for EventMoveVisualizer {
    fn start(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}

    fn is_finished(&self) -> MBool {
        assert!(self.current_move_index <= self.frames_count());
        self.current_move_index == self.frames_count()
    }

    fn draw(&mut self, geom: &Geom, scene: &mut Scene, state: &GameState) {
        let node_id = unit_id_to_node_id(self.unit_id);
        let node = scene.get_mut(&node_id);
        node.pos = self.current_position(geom, state);
        self.current_move_index += 1;
    }

    fn end(&mut self, geom: &Geom, scene: &mut Scene, state: &GameState) {
        let node_id = unit_id_to_node_id(self.unit_id);
        let unit_node = scene.get_mut(&node_id);
        unit_node.pos = self.current_position(geom, state);
    }
}

impl EventMoveVisualizer {
    pub fn new(unit_id: UnitId, path: ~[MapPos]) -> ~EventVisualizer {
        ~EventMoveVisualizer {
            unit_id: unit_id,
            path: path,
            current_move_index: 0,
        } as ~EventVisualizer
    }

    fn frames_count(&self) -> MInt {
        let len = self.path.len() as MInt - 1;
        (len * MOVE_SPEED as MInt) - 1
    }

    fn current_tile(&self) -> MapPos {
        self.path[self.current_tile_index()]
    }

    fn next_tile(&self) -> MapPos {
        self.path[self.current_tile_index() + 1]
    }

    fn current_tile_index(&self) -> MInt {
        self.current_move_index / MOVE_SPEED as MInt
    }

    fn node_index(&self) -> MInt {
        self.current_move_index - self.current_tile_index() * MOVE_SPEED as MInt
    }

    fn current_position(&self, geom: &Geom, state: &GameState) -> WorldPos {
        let unit_id = self.unit_id;
        let from = unit_pos(unit_id, self.current_tile(), geom, state);
        let to = unit_pos(unit_id, self.next_tile(), geom, state);
        let diff = to.sub_v(&from).div_s(MOVE_SPEED);
        from.add_v(&diff.mul_s(self.node_index() as MFloat))
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

    fn draw(&mut self, _: &Geom, _: &mut Scene, _: &GameState) {}

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
        scene.insert(node_id, SceneNode{pos: world_pos});
    }

    fn is_finished(&self) -> MBool {
        self.anim_index == MOVE_SPEED as MInt
    }

    fn draw(&mut self, geom: &Geom, scene: &mut Scene, state: &GameState) {
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

    fn draw(&mut self, _: &Geom, scene: &mut Scene, _: &GameState) {
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
