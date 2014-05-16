// See LICENSE file for copyright and license details.

use core::types::UnitId;
use visualizer::scene::{NodeId, Scene, SceneNode};
use core::game_state::GameState;
use visualizer::geom::{Geom, unit_pos};
use visualizer::mesh::MeshId;
use visualizer::types::WorldPos;

pub struct SelectionManager {
    unit_id: Option<UnitId>,
    node_id: NodeId,
    mesh_id: MeshId,
}

impl SelectionManager {
    pub fn new(mesh_id: MeshId) -> SelectionManager {
        SelectionManager {
            unit_id: None,
            node_id: NodeId{id: 666 + 1}, // TODO
            mesh_id: mesh_id,
        }
    }

    fn set_unit_id(&mut self, unit_id: UnitId) {
        self.unit_id = Some(unit_id);
    }

    fn get_pos(&self, geom: &Geom, state: &GameState) -> WorldPos {
        let unit_id = self.unit_id.unwrap();
        let map_pos = state.units.get(&unit_id).pos;
        let mut world_pos = unit_pos(unit_id, map_pos, geom, state);
        world_pos.v.z += 0.1; // TODO: replace with some constant
        world_pos
    }

    pub fn move_selection_marker(
        &self,
        geom: &Geom,
        state: &GameState,
        scene: &mut Scene
    ) {
        let node = scene.nodes.get_mut(&self.node_id);
        node.pos = self.get_pos(geom, state);
    }

    pub fn create_selection_marker(
        &mut self,
        geom: &Geom,
        state: &GameState,
        scene: &mut Scene,
        unit_id: UnitId
    ) {
        self.set_unit_id(unit_id);
        if scene.nodes.find(&self.node_id).is_some() {
            scene.nodes.remove(&self.node_id);
        }
        let node = SceneNode {
            pos: self.get_pos(geom, state),
            rot: 0.0,
            mesh_id: self.mesh_id,
        };
        scene.nodes.insert(self.node_id, node);
    }

    pub fn deselect(&mut self, scene: &mut Scene) {
        scene.nodes.remove(&self.node_id);
        self.unit_id = None;
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
