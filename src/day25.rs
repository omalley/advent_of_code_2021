#[derive(Clone,Copy,Debug)]
struct Location {
  x: usize,
  y: usize,
}

#[derive(Clone,Debug)]
pub struct State {
  is_occupied: Vec<Vec<bool>>,
  east_facing: Vec<Location>,
  south_facing: Vec<Location>,
  width: usize,
  height: usize,
}

impl State {
  fn parse(input: &mut dyn Iterator<Item = String>) -> State {
    let mut result = State{is_occupied: Vec::new(), east_facing: Vec::new(),
                           south_facing: Vec::new(), width: 0, height: 0};
    let mut first = true;
    for line in input {
      if first {
        first = false;
        result.width = line.chars().count();
      }
      result.is_occupied.push(vec![false; result.width]);
      let y = result.height;
      let mut x = 0;
      for ch in line.chars() {
        match ch {
          '>' => {
            result.is_occupied[y][x] = true;
            result.east_facing.push(Location{x, y});
          }
          'v' => {
            result.is_occupied[y][x] = true;
            result.south_facing.push(Location{x, y});
          }
          _ => {}
        }
        x += 1;
      }
      result.height += 1;
    }
    result
  }

  fn move_east(&mut self) -> usize {
    let mut moved: Vec<usize> = Vec::with_capacity(self.east_facing.len());
    for i in 0..self.east_facing.len() {
      let posn = self.east_facing[i];
      if !self.is_occupied[posn.y][(posn.x + 1) % self.width] {
        moved.push(i);
      }
    }
    for i in &moved {
      let old_posn = self.east_facing[*i];
      let new_posn = Location{x: (old_posn.x + 1) % self.width, y: old_posn.y };
      self.east_facing[*i] = new_posn;
      self.is_occupied[old_posn.y][old_posn.x] = false;
      self.is_occupied[new_posn.y][new_posn.x] = true;
    }
    moved.len()
  }

  fn move_south(&mut self) -> usize {
    let mut moved: Vec<usize> = Vec::with_capacity(self.south_facing.len());
    for i in 0..self.south_facing.len() {
      let posn = &self.south_facing[i];
      if !self.is_occupied[(posn.y + 1) % self.height][posn.x] {
        moved.push(i);
      }
    }
    for i in &moved {
      let old_posn = self.south_facing[*i];
      let new_posn = Location{x: old_posn.x, y: (old_posn.y + 1) % self.height};
      self.south_facing[*i] = new_posn;
      self.is_occupied[old_posn.y][old_posn.x] = false;
      self.is_occupied[new_posn.y][new_posn.x] = true;
    }
    moved.len()
  }
}

pub fn generator(input: &str) -> State {
  State::parse(&mut input.lines()
    .map(|x| x.trim().to_string())
    .filter(|x| x.len() > 0))
}

pub fn part1(input: &State) -> i32 {
  let mut state = input.clone();
  let mut turn = 1;
  while state.move_east() + state.move_south() > 0 {
    turn += 1;
  }
  turn
}

pub fn part2(_: &State) -> i32 {
  0
}