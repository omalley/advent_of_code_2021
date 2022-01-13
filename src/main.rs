use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let mut octo = Octopus::parse(&mut stdin.lock().lines()
                                 .map(|x| String::from(x.unwrap().trim()))
                                 .filter(|x| x.len() > 0));

  loop {
    if octo.advance() == octo.width * octo.energy.len() {
      break;
    }
  }
  println!("turns = {}", octo.turn);
}

#[derive(Debug,Default)]
struct Octopus {
  energy: Vec<Vec<u32>>,
  width: usize,
  turn: u64,
}

#[derive(Clone,Copy,Debug)]
struct Point {
  x: usize,
  y: usize,
}

const OCTOPUS_RADIX: u32 = 10;

impl Octopus {
  fn parse(input: &mut dyn Iterator<Item = String>) -> Self {
    let mut result = Octopus::default();
    result.energy = input.map(|line| line.chars()
                                .map(|c| c.to_digit(OCTOPUS_RADIX).unwrap())
                                .collect())
                         .collect();
    result.width =
      match result.energy.iter().map(|x| x.len())
              .reduce(|a, b| usize::min(a, b)) {
        None => 0,
        Some(x) => x,
      };
    result
  }

  fn neighbors(&self, pnt: &Point) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    for relative_x in -1..=1 {
      for relative_y in -1..=1 {
        if relative_x != 0 || relative_y != 0 {
          let off_x = pnt.x as i64 + relative_x;
          let off_y = pnt.y as i64 + relative_y;
          if off_x >= 0 && off_x < self.width as i64 &&
             off_y >= 0 && off_y < self.energy.len() as i64 {
            result.push(Point{x: off_x as usize, y: off_y as usize});
          }
        }
      }
    }
    result
  }
  
  fn advance(&mut self) -> usize {
    let mut to_do: Vec<Point> = Vec::new();
    for x in 0..self.width {
      for y in 0..self.energy.len() {
        to_do.push(Point{x : x, y: y});
      }
    }

    // update all of the squares
    while to_do.len() > 0 {
      let p = to_do.pop().unwrap();
      self.energy[p.y][p.x] += 1;
      // if it went to 10, bump up the neighbors again
      if self.energy[p.y][p.x] == OCTOPUS_RADIX {
        to_do.extend(self.neighbors(&p).iter());
      }
    }

    self.turn += 1;
    
    let mut lights = 0;
    for x in 0..self.width {
      for y in 0..self.energy.len() {
        if self.energy[y][x] >= OCTOPUS_RADIX {
          self.energy[y][x] = 0;
          lights += 1;
        }
      }
    }
    lights
  }
}