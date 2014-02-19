// See LICENSE file for copyright and license details.

use std::hashmap::HashMap;
use core_types::{
  Size2,
  Int,
  UnitId,
  PlayerId,
  MapPos,
};

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
  map_size: Size2<Int>,
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
      map_size: Size2{x: 4, y: 8},
    }
  }

  pub fn get_event_view(&mut self) -> Option<EventView> {
    let list = self.event_view_lists.get_mut(&self.current_player_id);
    list.shift()
  }

  fn id_to_unit_mut_opt(&'a mut self, id: UnitId) -> Option<&'a mut Unit> {
    self.units.mut_iter().find(|u| u.id == id)
  }

  fn id_to_unit_mut(&'a mut self, id: UnitId) -> &'a mut Unit {
    match self.id_to_unit_mut_opt(id) {
      Some(unit) => unit,
      None => fail!("Bad unit id: {}", id),
    }
  }

  fn id_to_unit_opt(&'a self, id: UnitId) -> Option<&'a Unit> {
    self.units.iter().find(|u| u.id == id)
  }

  fn id_to_unit(&'a self, id: UnitId) -> &'a Unit {
    match self.id_to_unit_opt(id) {
      Some(unit) => unit,
      None => fail!("Bad unit id: {}", id),
    }
  }

  // TODO: Remove or make private
  pub fn unit_at_opt(&'a self, pos: MapPos) -> Option<&'a Unit> {
    self.units.iter().find(|u| u.pos == pos)
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

  fn make_event_views(&mut self) {
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
  // TODO: fn is_visible(&self) -> Bool;
}

struct EventMove {
  unit_id: UnitId,
  path: ~[MapPos],
}

impl EventMove {
  fn new(core: &Core, unit_id: UnitId, destination: MapPos) -> ~EventMove {
    let start_pos = core.id_to_unit(unit_id).pos;
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
    let unit = core.id_to_unit_mut(self.unit_id);
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
    let max_id = core.players.len() as Int;
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
