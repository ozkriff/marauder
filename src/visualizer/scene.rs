// See LICENSE file for copyright and license details.

use crate::core::types::MInt;
use crate::visualizer::mesh::MeshId;
use crate::visualizer::types::{MFloat, WorldPos};
use std::collections::HashMap;

// TODO: why scene knows about other systems?
pub const MAX_UNIT_NODE_ID: NodeId = NodeId { id: 1000 };
pub const MIN_MARKER_NODE_ID: NodeId = NodeId {
    id: MAX_UNIT_NODE_ID.id + 1,
};
pub const MAX_MARKER_NODE_ID: NodeId = NodeId {
    id: MAX_UNIT_NODE_ID.id * 2,
};
pub const SHELL_NODE_ID: NodeId = NodeId {
    id: MAX_MARKER_NODE_ID.id + 1,
};
pub const SELECTION_NODE_ID: NodeId = NodeId {
    id: SHELL_NODE_ID.id + 1,
};

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct NodeId {
    pub id: MInt,
}

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
