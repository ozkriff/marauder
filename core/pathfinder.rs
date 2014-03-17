// See LICENSE file for copyright and license details.

use std::vec_ng::Vec;
use core::types::{MBool, MInt, MapPos, Size2};
use core::core::Unit;
use core::game_state::GameState;
use core::dir::Dir;

struct Tile {
    cost: MInt,
    parent: Option<Dir>,
}

struct Map {
    size: Size2<MInt>,
    tiles: Vec<Tile>,
}

fn max_cost() -> MInt {
    30000
}

impl<'a> Map {
    fn tile_mut(&'a mut self, pos: MapPos) -> &'a mut Tile {
        self.tiles.get_mut((pos.x + pos.y * self.size.w) as uint)
    }

    fn tile(&'a self, pos: MapPos) -> &'a Tile {
        self.tiles.get((pos.x + pos.y * self.size.w) as uint)
    }

    fn is_inboard(&self, pos: MapPos) -> MBool {
        let x = pos.x;
        let y = pos.y;
        x >= 0 && y >= 0 && x < self.size.w && y < self.size.h
    }
}

pub struct Pathfinder {
    priv queue: Vec<MapPos>,
    priv map: Map,
}

fn create_tiles(tiles_count: MInt) -> Vec<Tile> {
    let mut tiles = Vec::new();
    for _ in range(0, tiles_count) {
        tiles.push(Tile {
            cost: 0,
            parent: None,
        });
    }
    tiles
}

impl Pathfinder {
    pub fn new(map_size: Size2<MInt>) -> Pathfinder {
        let tiles_count = map_size.w * map_size.h;
        Pathfinder {
            queue: Vec::new(),
            map: Map {
                size: map_size,
                tiles: create_tiles(tiles_count),
            },
        }
    }

    fn process_neighbour_pos(
        &mut self,
        _: &GameState,
        _: &Unit, // TODO: use unit`s type and action points later
        original_pos: MapPos,
        neighbour_pos: MapPos
    ) {
        let old_cost = self.map.tile(original_pos).cost;
        let tile = self.map.tile_mut(neighbour_pos);
        let new_cost: MInt = old_cost + 1;
        if tile.cost > new_cost {
            self.queue.push(neighbour_pos);
            // update neighbour tile info
            tile.cost = new_cost;
            let dir = Dir::get_dir_from_to(neighbour_pos, original_pos);
            tile.parent = Some(dir);
        }
    }

    fn clean_map(&mut self) {
        for tile in self.map.tiles.mut_iter() {
            tile.cost = max_cost();
            tile.parent = None;
        }
    }

    fn try_to_push_neighbours(
        &mut self,
        state: &GameState,
        unit: &Unit,
        pos: MapPos
    ) {
        assert!(self.map.is_inboard(pos));
        for i in range(0 as MInt, 6) {
            let neighbour_pos = Dir::get_neighbour_pos(pos, Dir::from_int(i));
            if self.map.is_inboard(neighbour_pos) {
                self.process_neighbour_pos(
                    state, unit, pos, neighbour_pos);
            }
        }
    }

    fn push_start_pos_to_queue(&mut self, start_pos: MapPos) {
        self.queue.push(start_pos);
        let start_tile = self.map.tile_mut(start_pos);
        start_tile.cost = 0;
        start_tile.parent = None;
    }

    pub fn fill_map(&mut self, state: &GameState, unit: &Unit) {
        assert!(self.queue.len() == 0);
        self.clean_map();
        self.push_start_pos_to_queue(unit.pos);
        while self.queue.len() != 0 {
            let pos = self.queue.shift().unwrap();
            self.try_to_push_neighbours(state, unit, pos);
        }
    }

    pub fn get_path(&self, destination: MapPos) -> Vec<MapPos> {
        let mut path = Vec::new();
        let mut pos = destination;
        assert!(self.map.is_inboard(pos));
        path.push(destination);
        while self.map.tile(pos).cost != 0 {
            let parent_dir = self.map.tile(pos).parent.unwrap();
            pos = Dir::get_neighbour_pos(pos, parent_dir);
            assert!(self.map.is_inboard(pos));
            path.push(pos);
        }
        path.reverse();
        path
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
