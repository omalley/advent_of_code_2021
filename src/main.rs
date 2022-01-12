use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let map = Map::parse(&mut stdin.lock().lines()
                              .map(|x| String::from(x.unwrap().trim()))
                              .filter(|x| x.len() > 0));
  let mut risk = 0;
  for x in 0..map.get_width() {
    for y in 0..map.get_height() {
      if map.is_low(x, y) {
        risk += 1 + map.get_elevation(x, y)
      }
    }
  }
  println!("risk = {}", risk);
}

#[derive(Debug,Default)]
struct Map {
  elevation: Vec<Vec<u32>>,
  width: usize,
}

const MAP_RADIX: u32 = 10;

impl Map {
  fn parse(input: &mut dyn Iterator<Item = String>) -> Self {
    let mut result = Map::default();
    result.elevation = input
        .map(|x| x.chars().map(|c| c.to_digit(MAP_RADIX).unwrap()).collect())
        .collect();
    if result.elevation.len() != 0 {
      result.width = result.elevation[0].len();
    }
    result
  }

  fn get_width(&self) -> usize {
    self.width
  }

  fn get_height(&self) -> usize {
    self.elevation.len()
  }
  
  fn get_elevation(&self, x: usize, y: usize) -> u32 {
    self.elevation[y][x]
  }

  fn get_left(&self, x: usize, y: usize) -> Option<u32> {
    if x == 0 {
      None
    } else {
      Some(self.get_elevation(x - 1, y))
    }
  }
  
  fn get_right(&self, x: usize, y: usize) -> Option<u32> {
    if x == self.get_width() - 1 {
      None
    } else {
      Some(self.get_elevation(x + 1, y))
    }
  }
  
  fn get_up(&self, x: usize, y: usize) -> Option<u32> {
    if y == 0 {
      None
    } else {
      Some(self.get_elevation(x, y - 1))
    }
  }
  
  fn get_down(&self, x: usize, y: usize) -> Option<u32> {
    if y == self.get_height() - 1 {
      None
    } else {
      Some(self.get_elevation(x, y + 1))
    }
  }
  
  fn is_low(&self, x: usize, y: usize) -> bool {
    // get the elevations of the neighbors
    let mut neighbors = Vec::new();
    neighbors.extend(self.get_left(x, y).into_iter());
    neighbors.extend(self.get_right(x, y).into_iter());
    neighbors.extend(self.get_up(x, y).into_iter());
    neighbors.extend(self.get_down(x, y).into_iter());

    // get the minimum
    let min = neighbors.iter().fold(u32::MAX, |acc, x| u32::min(acc, *x));
    self.get_elevation(x, y) < min
  }
}
