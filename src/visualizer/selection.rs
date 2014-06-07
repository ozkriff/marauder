// See LICENSE file for copyright and license details.

use cgmath::vector::Vector2;
use core::types::{MInt, UnitId};
use core::game_state::GameState;
use core::misc::add_quad_to_vec;
use core::fs::FileSystem;
use visualizer::scene::{NodeId, Scene, SceneNode};
use visualizer::geom;
use visualizer::mesh::{Mesh, MeshId};
use visualizer::texture::Texture;
use visualizer::types::{WorldPos, TextureCoord};
use visualizer::shader::Shader;

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

    fn get_pos(&self, state: &GameState) -> WorldPos {
        let unit_id = self.unit_id.unwrap();
        let map_pos = state.units.get(&unit_id).pos;
        let mut world_pos = geom::map_pos_to_world_pos(map_pos);
        world_pos.v.z += 0.1; // TODO: replace with some constant
        world_pos
    }

    pub fn move_selection_marker(
        &self,
        state: &GameState,
        scene: &mut Scene
    ) {
        let node = scene.nodes.get_mut(&self.node_id);
        node.pos = self.get_pos(state);
    }

    pub fn create_selection_marker(
        &mut self,
        state: &GameState,
        scene: &mut Scene,
        unit_id: UnitId
    ) {
        self.set_unit_id(unit_id);
        if scene.nodes.find(&self.node_id).is_some() {
            scene.nodes.remove(&self.node_id);
        }
        let node = SceneNode {
            pos: self.get_pos(state),
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

pub fn get_selection_mesh(fs: &FileSystem, shader: &Shader) -> Mesh {
    let tex = Texture::new(&fs.get(&Path::new("data/shell.png")));
    let mut vertex_data = Vec::new();
    let mut tex_data = Vec::new();
    let scale_1 = 0.4;
    let scale_2 = scale_1 + 0.05;
    for num in range(0 as MInt, 6) {
        let vertex_1_1 = geom::index_to_hex_vertex_s(scale_1, num);
        let vertex_1_2 = geom::index_to_hex_vertex_s(scale_2, num);
        let vertex_2_1 = geom::index_to_hex_vertex_s(scale_1, num + 1);
        let vertex_2_2 = geom::index_to_hex_vertex_s(scale_2, num + 1);
        add_quad_to_vec(
            &mut vertex_data,
            vertex_2_1,
            vertex_2_2,
            vertex_1_2,
            vertex_1_1,
        );
        add_quad_to_vec(
            &mut tex_data,
            TextureCoord{v: Vector2{x: 0.0, y: 0.0}},
            TextureCoord{v: Vector2{x: 0.0, y: 1.0}},
            TextureCoord{v: Vector2{x: 1.0, y: 1.0}},
            TextureCoord{v: Vector2{x: 1.0, y: 0.0}},
        );
    }
    let mut mesh = Mesh::new(vertex_data.as_slice());
    mesh.set_texture(tex, tex_data.as_slice());
    mesh.prepare(shader);
    mesh
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
