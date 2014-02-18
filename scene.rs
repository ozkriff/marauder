// See LICENSE file for copyright and license details.

use std::hashmap::HashMap;
use world_pos::WorldPos;
use core_types::Int;

pub struct SceneNode {
  pos: WorldPos,
  // rot: Angle,
}

pub type NodeId = Int;
pub type Scene = HashMap<NodeId, SceneNode>;

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
