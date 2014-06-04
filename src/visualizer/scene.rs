// See LICENSE file for copyright and license details.

use core::types::{MInt};
use visualizer::types::{WorldPos, MFloat};
use visualizer::mesh::MeshId;
use collections::hashmap::HashMap;

#[deriving(PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct NodeId{pub id: MInt}

pub struct SceneNode {
    pub pos: WorldPos,
    pub rot: MFloat,
    pub mesh_id: MeshId,
}

pub struct Scene {
    pub nodes: HashMap<NodeId, SceneNode>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            nodes: HashMap::new(),
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
