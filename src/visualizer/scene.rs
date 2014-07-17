// See LICENSE file for copyright and license details.

use core::types::{MInt};
use visualizer::types::{WorldPos, MFloat};
use visualizer::mesh::MeshId;
use std::collections::hashmap::HashMap;

// TODO: why scene knows about other systems?
pub static MAX_UNIT_NODE_ID: NodeId = NodeId{id: 1000};
pub static SHELL_NODE_ID: NodeId = NodeId{id: MAX_UNIT_NODE_ID.id + 1};
pub static SELECTION_NODE_ID: NodeId = NodeId{id: SHELL_NODE_ID.id + 1};

#[deriving(PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct NodeId{pub id: MInt}

pub struct SceneNode {
    pub pos: WorldPos,
    pub rot: MFloat,
    pub mesh_id: Option<MeshId>,
    pub children: Vec<SceneNode>,
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
