// See LICENSE file for copyright and license details.

use cgmath::vector::Vec2;

pub struct TileIterator {
  cursor: Vec2<i32>
}

impl TileIterator {
  pub fn new() -> TileIterator {
    TileIterator {
      cursor: Vec2::<i32>{x: 0, y: 0}
    }
  }
}

impl Iterator<Vec2<i32>> for TileIterator {
  fn next(&mut self) -> Option<Vec2<i32>> {
    let map_size = Vec2::<i32>{x: 3, y: 4};
    self.cursor.x += 1;
    if self.cursor.x == map_size.x {
      self.cursor.x = 0;
      self.cursor.y += 1;
    }
    if self.cursor.y == map_size.y {
      None
    } else {
      Some(self.cursor)
    }
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
