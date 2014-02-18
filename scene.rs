// See LICENSE file for copyright and license details.

use std::hashmap::HashMap;
use world_pos::WorldPos;

pub struct SceneNode {
  pos: WorldPos,
  // rot: Angle,
}

pub type NodeId = i32;
pub type Scene = HashMap<NodeId, SceneNode>;

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
