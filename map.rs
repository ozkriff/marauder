// See LICENSE file for copyright and license details.

use cgmath::vector::Vec2;

pub struct TileIterator {
  cursor: Vec2<i32>,
}

impl TileIterator {
  pub fn new() -> TileIterator {
    TileIterator {
      cursor: Vec2::zero(),
    }
  }
}

impl Iterator<Vec2<i32>> for TileIterator {
  fn next(&mut self) -> Option<Vec2<i32>> {
    let map_size = Vec2::<i32>{x: 3, y: 4};
    let current_pos = if self.cursor.y > map_size.y {
      None
    } else {
      Some(self.cursor)
    };
    self.cursor.x += 1;
    if self.cursor.x > map_size.x {
      self.cursor.x = 0;
      self.cursor.y += 1;
    }
    current_pos
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
