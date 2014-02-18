// See LICENSE file for copyright and license details.

use cgmath::vector::Vector;
use gl::types::GLfloat;
use geom::Geom;
use core::{
  MapPos,
  UnitId,
};
use scene::Scene;
use world_pos::WorldPos;

pub trait EventVisualizer {
  fn is_finished(&self) -> bool;
  fn draw(&mut self, geom: &Geom, scene: &mut Scene);
  fn end(&mut self, geom: &Geom, scene: &mut Scene);
}

static MOVE_SPEED: GLfloat = 40.0; // TODO: config?

pub struct EventMoveVisualizer {
  unit_id: UnitId,
  path: ~[MapPos],
  current_move_index: i32,
}

impl EventVisualizer for EventMoveVisualizer {
  fn is_finished(&self) -> bool {
    assert!(self.current_move_index <= self.frames_count());
    self.current_move_index == self.frames_count()
  }

  fn draw(&mut self, geom: &Geom, scene: &mut Scene) {
    let node = scene.get_mut(&self.unit_id);
    node.pos = self.current_position(geom);
    self.current_move_index += 1;
  }

  fn end(&mut self, geom: &Geom, scene: &mut Scene) {
    self.end_movement(geom, scene);
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

  fn frames_count(&self) -> i32 {
    let len = self.path.len() as i32 - 1;
    len * MOVE_SPEED as i32
  }

  fn current_tile(&self) -> MapPos {
    self.path[self.current_tile_index()]
  }

  fn next_tile(&self) -> MapPos {
    self.path[self.current_tile_index() + 1]
  }

  fn end_movement(&mut self, geom: &Geom, scene: &mut Scene) {
    let unit_node = scene.get_mut(&self.unit_id);
    unit_node.pos = self.current_position(geom);
  }

  fn current_tile_index(&self) -> i32 {
    // self.current_move_index / MOVE_SPEED as i32
    0
  }

  fn node_index(&self) -> i32 {
    // self.current_move_index - self.current_tile_index() * MOVE_SPEED
    self.current_move_index
  }

  fn current_position(&self, geom: &Geom) -> WorldPos {
    let from = geom.map_pos_to_world_pos(self.current_tile());
    let to = geom.map_pos_to_world_pos(self.next_tile());
    let diff = to.sub_v(&from).div_s(MOVE_SPEED);
    from.add_v(&diff.mul_s(self.node_index() as GLfloat))
  }
}

pub struct EventEndTurnVisualizer;

impl EventEndTurnVisualizer {
  pub fn new() -> ~EventVisualizer {
    ~EventEndTurnVisualizer as ~EventVisualizer
  }
}

impl EventVisualizer for EventEndTurnVisualizer {
  fn is_finished(&self) -> bool {
    true
  }

  fn draw(&mut self, _: &Geom, _: &mut Scene) {}

  fn end(&mut self, _: &Geom, _: &mut Scene) {}
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
