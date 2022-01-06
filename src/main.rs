use std::io;
use std::io::BufRead;

fn main() {
  // Get the input lines
  let lines: Vec<String> = io::stdin().lock().lines()
    .map(|x| String::from(x.unwrap().trim())).collect();
  let cmds: Vec<Move> = lines.iter().map(|x| parse(x)).collect();
  let mut posn = Position{x: 0, y: 0};
  for c in cmds {
    posn.update(&c);
  }
  println!("area = {}", posn.area())
}

enum Move {
  Up(i32),
  Down(i32),
  Forward(i32),
}

fn parse(s: &String) -> Move {
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

struct Position {
  x: i32,
  y: i32
}

impl Position {
  fn update(self: &mut Position, m: &Move) {
    match m {
      Move::Up(i) => self.y -= i,
      Move::Down(i) => self.y += i,
      Move::Forward(i) => self.x += i,
    }
  }

  fn area(self: &Position) -> i32 {
    self.x * self.y
  }
}
