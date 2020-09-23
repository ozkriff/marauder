// See LICENSE file for copyright and license details.

use crate::core::core::UnitTypeId;
use crate::visualizer::mesh::MeshId;
use crate::visualizer::types::MFloat;

pub struct UnitTypeVisualInfo {
    pub mesh_id: MeshId,
    pub move_speed: MFloat, // TODO: MFloat -> Speed
}

pub struct UnitTypeVisualInfoManager {
    list: Vec<UnitTypeVisualInfo>,
}

impl UnitTypeVisualInfoManager {
    pub fn new() -> UnitTypeVisualInfoManager {
        UnitTypeVisualInfoManager { list: Vec::new() }
    }

    pub fn add_info(&mut self, info: UnitTypeVisualInfo) {
        self.list.push(info);
    }

    pub fn get(&self, type_id: UnitTypeId) -> &UnitTypeVisualInfo {
        &self.list[type_id.id as usize]
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
