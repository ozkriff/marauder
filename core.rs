// See LICENSE file for copyright and license details.

use std::hashmap::HashMap;
use cgmath::vector::Vec2;

pub type PlayerId = i32;
pub type UnitId = i32;
pub type MapPos = Vec2<i32>;

pub enum Command {
  CommandMove(UnitId, MapPos),
  CommandEndTurn,
}

pub enum EventView {
  EventViewMove(UnitId, ~[MapPos]),
  EventViewEndTurn(PlayerId, PlayerId), // old_id, new_id
}

pub struct Player {
  id: PlayerId,
}

pub struct Unit {
  id: UnitId,
  pos: MapPos,
}

pub struct Core<'a> {
  units: ~[Unit],
  players: ~[Player],
  current_player_id: PlayerId,
  event_list: ~[~Event],
  event_view_lists: HashMap<PlayerId, ~[EventView]>,
}

fn get_event_view_lists() -> HashMap<PlayerId, ~[EventView]> {
  let mut map = HashMap::new();
  map.insert(0 as PlayerId, ~[]);
  map.insert(1 as PlayerId, ~[]);
  map
}

impl<'a> Core<'a> {
  pub fn new() -> Core {
    Core {
      units: ~[],
      players: ~[Player{id: 0}, Player{id: 1}],
      current_player_id: 0,
      event_list: ~[],
      event_view_lists: get_event_view_lists(),
    }
  }

  pub fn id_to_unit_mut(&'a mut self, id: UnitId) -> Option<&'a mut Unit> {
    // TODO: Simplify
    for unit in self.units.mut_iter() {
      if unit.id == id {
        return Some(unit);
      }
    }
    None
  }

  pub fn id_to_unit(&'a self, id: UnitId) -> Option<&'a Unit> {
    for unit in self.units.iter() {
      if unit.id == id {
        return Some(unit);
      }
    }
    None
  }

  fn command_to_event(&self, command: Command) -> ~Event {
    match command {
      CommandEndTurn => {
        EventEndTurn::new(self) as ~Event
      },
      CommandMove(unit_id, destination) => {
        EventMove::new(self, unit_id, destination) as ~Event
      },
    }
  }

  pub fn do_command(&mut self, command: Command) {
    let event = self.command_to_event(command);
    self.event_list.push(event);
    self.make_event_views();
  }

  pub fn make_event_views(&mut self) {
    while self.event_list.len() != 0 {
      let event = self.event_list.pop().unwrap();
      event.apply(self);
      for player in self.players.iter() {
        let event_view_list = self.event_view_lists.get_mut(&player.id);
        event_view_list.push(event.get_view());
      }
    }
  }
}

trait Event {
  fn apply(&self, core: &mut Core);
  fn get_view(&self) -> EventView;
  // TODO: fn is_visible(&self) -> bool;
}

struct EventMove {
  unit_id: UnitId,
  path: ~[MapPos],
}

impl EventMove {
  fn new(core: &Core, unit_id: UnitId, destination: MapPos) -> ~EventMove {
    let start_pos = match core.id_to_unit(unit_id) {
      Some(unit) => unit.pos,
      None => fail!("Bad unit id: {}", unit_id),
    };
    ~EventMove {
      path: ~[start_pos, destination],
      unit_id: unit_id,
    }
  }
}

impl Event for EventMove {
  fn get_view(&self) -> EventView {
    EventViewMove(self.unit_id, self.path.clone())
  }

  fn apply(&self, core: &mut Core) {
    let unit = match core.id_to_unit_mut(self.unit_id) {
      Some(unit) => unit,
      None => fail!("Bad unit id: {}", self.unit_id),
    };
    unit.pos = *self.path.last().unwrap();
  }
}

struct EventEndTurn {
  old_id: PlayerId,
  new_id: PlayerId,
}

impl EventEndTurn {
  fn new(core: &Core) -> ~EventEndTurn {
    let old_id = core.current_player_id;
    let max_id = core.players.len() as i32;
    let new_id = if old_id + 1 == max_id { 0 } else { old_id + 1 };
    ~EventEndTurn{old_id: old_id, new_id: new_id}
  }
}

impl Event for EventEndTurn {
  fn get_view(&self) -> EventView {
    EventViewEndTurn(self.old_id, self.new_id)
  }

  fn apply(&self, core: &mut Core) {
    // core.deselected_any_units();
    for player in core.players.iter() {
      if player.id == self.new_id {
        if core.current_player_id == self.old_id {
          core.current_player_id = player.id;
        }
        return;
      }
    }
  }
}

// vim: set tabstop=2 shiftwidth=2 softtabstop=2 expandtab:
