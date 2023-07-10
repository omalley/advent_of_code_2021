use std::cmp::Reverse;
use std::fmt;
use std::fmt::Formatter;

use priority_queue::priority_queue::PriorityQueue;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, PartialEq)]
enum AmphipodKind {
  Amber = 0,
  Bronze = 1,
  Copper = 2,
  Desert = 3,
}

impl AmphipodKind {
  fn energy(&self) -> usize {
    match self {
      Self::Amber => 1,
      Self::Bronze => 10,
      Self::Copper => 100,
      Self::Desert => 1000,
    }
  }

  fn parse(s: char) -> Option<Self> {
    match s {
      'A' => Some(Self::Amber),
      'B' => Some(Self::Bronze),
      'C' => Some(Self::Copper),
      'D' => Some(Self::Desert),
      _ => None,
    }
  }

  fn name(&self) -> &str {
    match self {
      Self::Amber => "A",
      Self::Bronze => "B",
      Self::Copper => "C",
      Self::Desert => "D",
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Amphipod {
  kind: AmphipodKind,
  spot: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]

struct State {
  energy: usize,
  amphipods: Vec<Amphipod>,
}

impl State {
  // Get the bit vector of which rooms are occupied
  fn get_occupied(&self) -> u64 {
    let mut result: u64 = 0;
    for a in &self.amphipods {
      result |= 1 << a.spot;
    }
    result
  }
}

impl fmt::Display for State {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "energy: {} [", self.energy)?;
    for a in &self.amphipods {
      write!(f, "{}: {}, ", a.kind.name(), a.spot)?;
    }
    write!(f, "]")
  }
}

#[derive(Debug)]
struct Path {
  dest: usize,
  length: usize,
  blocked_by: u64,
}

#[derive(Debug)]
struct Spot {
  id: usize,
  x: usize,
  y: usize,
  is_home: Option<AmphipodKind>,
  exits: Vec<Path>,
}

fn is_between(x0: usize, x1: usize, p: usize) -> bool {
  usize::min(x0, x1) <= p && p <= usize::max(x0, x1)
}

impl Spot {
  fn manhattan(&self, other: &Self) -> usize {
    (i64::abs(self.x as i64 - other.x as i64) +
        i64::abs(self.y as i64 - other.y as i64)) as usize
  }

  fn is_blocking(&self, source: &Self, dest: &Self) -> bool {
    let hallway = if source.is_home.is_none() {source} else {dest};
    let room = if source.is_home.is_none() {dest} else {source};
    if self.is_home.is_none() {
      is_between(hallway.x, room.x, self.x)
    } else {
      self.x == room.x && self.y < room.y
    }
  }
}

#[derive(Debug)]
struct Caves {
  spots: Vec<Spot>,
  initial: State,
  // kind -> list of room ids
  goals: Vec<Vec<usize>>,
}

impl Caves {
  fn parse(lines: &Vec<String>) -> Self {
    let mut spots: Vec<Spot> = Vec::new();
    let mut amphipods: Vec<Amphipod> = Vec::new();
    let mut goals: Vec<Vec<usize>> = vec![Vec::new(); AmphipodKind::iter().len()];
    // assume the shape is still the same
    let hallway: Vec<char>  = lines[1].chars().collect();
    let rooms: Vec<char> = lines[2].chars().collect();
    for x in 0..hallway.len() {
      if hallway[x] == '.' && rooms[x] == '#' {
        let id = spots.len();
        spots.push(Spot{id, x, y:1, is_home: None, exits: Vec::new()});
      }
    }
    for y in 2..lines.len() {
      let rooms: Vec<char> = lines[y].chars().collect();
      let mut kind_itr = AmphipodKind::iter();
      let mut room_num = 0;
      for x in 0..rooms.len() {
        if rooms[x] != '#' && rooms[x] != ' ' {
          let kind = kind_itr.next();
          let id = spots.len();
          let spot = Spot{id, x, y, is_home: kind, exits: Vec::new()};
          goals[room_num].insert(0, id);
          if let Some(occupant) = AmphipodKind::parse(rooms[x]) {
            amphipods.push(Amphipod{kind: occupant, spot: spot.id});
          }
          spots.push(spot);
          room_num += 1;
        }
      }
    }
    for s in 0..spots.len() {
      for e in Self::build_edges(s, &spots) {
        spots[s].exits.push(e);
      }
    }
    Caves{spots,
          initial: State {
            energy: 0,
            amphipods: amphipods.as_slice().try_into().unwrap()},
          goals
    }
  }

  fn build_edges(from_idx: usize, spots: &Vec<Spot>) -> Vec<Path> {
    let mut result: Vec<Path> = Vec::new();
    let from = &spots[from_idx];
    for dest in spots {
      // Only make edges from the hallway to rooms & back
      if dest.is_home.is_none() != from.is_home.is_none() {
        let mut blockers: u64 = 1 << dest.id;
        for block in spots {
          if block.id != dest.id && block.id != from.id &&
             block.is_blocking(from, dest) {
            blockers |= 1 << block.id;
          }
        }
        result.push(Path{dest: dest.id, length: from.manhattan(dest),
          blocked_by: blockers});
      }
    }
    result
  }

  fn analyze(&self, state: &State) -> AnalyzedState {
    let mut occupant: Vec<Option<usize>> = vec![None; self.spots.len()];
    for (a_idx, a) in state.amphipods.iter().enumerate() {
      occupant[a.spot] = Some(a_idx);
    }
    let mut is_done= vec![false; state.amphipods.len()];
    let mut blocked: u64 = 0;
    for goals in &self.goals {
      let mut right_color = true;
      for &room in goals {
        if right_color {
          if let Some(occ) = occupant[room] {
            if right_color && state.amphipods[occ].kind == self.spots[room].is_home.unwrap() {
              is_done[occ] = true;
            } else {
              right_color = false;
            }
          } else {
            right_color = false;
          }
        } else {
          blocked |= 1 << room;
        }
      }
    }
    AnalyzedState{is_done, blocked}
  }
}

#[derive (Debug)]
struct AnalyzedState {
  // is a given amphipod in their final place
  is_done: Vec<bool>,
  // is a given room spot blocked because a wrong
  // color amphipod is in the room
  blocked: u64,
}

impl AnalyzedState {
  // which amphipods still need to move
  fn remaining(&self) -> Vec<usize> {
    self.is_done.iter().enumerate()
        .filter_map(|(i, &done)| if !done { Some(i) } else { None }).collect()
  }

  fn is_all_done(&self) -> bool {
    self.is_done.iter().fold(true, |a, &b| a && b)
  }
}

fn find_best_solution(input: &Vec<String>) -> usize {
  let caves = Caves::parse(input);
  let mut to_do: PriorityQueue<State, Reverse<usize>> = PriorityQueue::new();
  to_do.push(caves.initial.clone(), Reverse(caves.initial.energy));
  while let Some((current, _)) = to_do.pop() {
    let analyzed = caves.analyze(&current);
    if analyzed.is_all_done() {
      return current.energy
    }
    let occupied = current.get_occupied();
    for i in analyzed.remaining() {
      let amphipod = current.amphipods[i];
      for exit in caves.spots[amphipod.spot].exits.iter()
        .filter(|&e| (e.blocked_by & occupied == 0) &&
          (1 << e.dest & analyzed.blocked == 0)) {
        match caves.spots[exit.dest].is_home {
          Some(a) => if a != amphipod.kind { continue }
          None => {}
        }
        let mut next = current.clone();
        let next_energy = current.energy + exit.length * amphipod.kind.energy();
        next.energy = next_energy;
        next.amphipods[i].spot = exit.dest;
        to_do.push(next, Reverse(next_energy));
      }
    }
  }
  panic!("Can't find solution")
}

pub fn generator(input: &str) -> Vec<String> {
  input.lines().filter(|&x| x.len() > 0)
    .map(|x| x.to_string()).collect()
}

pub fn part1(input: &Vec<String>) -> usize {
  find_best_solution(input)
}

pub fn part2(input: &Vec<String>) -> usize {
  let mut modified_input = input.clone();
  modified_input.insert(3, "  #D#C#B#A#  ".to_string());
  modified_input.insert(4, "  #D#B#A#C#  ".to_string());
  find_best_solution(&modified_input)
}
