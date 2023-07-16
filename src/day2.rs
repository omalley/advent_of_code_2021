pub enum Move {
  Up(i32),
  Down(i32),
  Forward(i32),
}

impl Move {
  fn parse(s: &str) -> Move {
    let mut parts = s.split_whitespace();
    let command = parts.next().unwrap();
    let dist = parts.next().unwrap().parse::<i32>().unwrap();
    match command {
      "forward" => Move::Forward(dist),
      "up" => Move::Up(dist),
      "down" => Move::Down(dist),
      _ => panic!("Unknown command {}", command),
    }
  }
}

struct Position {
  x: i32,
  y: i32,
  aim: i32,
}

impl Position {
  fn part1_update(self: &mut Position, m: &Move) {
    match m {
      Move::Up(i) => self.y -= i,
      Move::Down(i) => self.y += i,
      Move::Forward(i) => self.x += i,
    }
  }

  fn part2_update(self: &mut Position, m: &Move) {
    match m {
      Move::Up(i) => self.aim -= i,
      Move::Down(i) => self.aim += i,
      Move::Forward(i) => {
        self.x += i;
        self.y += self.aim * i;
      }
    }
  }

  fn area(self: &Position) -> i32 {
    self.x * self.y
  }
}

pub fn generator(data: &str) -> Vec<Move> {
  data.lines()
    .map(|x| Move::parse(x.trim()))
    .collect()
}

pub fn part1(cmds: &Vec<Move>) -> i32 {
  let mut posn = Position{x: 0, y: 0, aim: 0};
  for c in cmds {
    posn.part1_update(&c);
  }
  posn.area()
}

pub fn part2(cmds: &Vec<Move>) -> i32 {
  let mut posn = Position{x: 0, y: 0, aim: 0};
  for c in cmds {
    posn.part2_update(&c);
  }
  posn.area()
}

