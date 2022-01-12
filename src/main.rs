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

  let mut basin_sizes = map.find_basins();
  basin_sizes.sort_by(|a, b| b.cmp(a));
  let prod = basin_sizes[0..3].iter().fold(1, |a,b| a * b);
  println!("sizes = {:?}, product = {}", basin_sizes, prod);
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

  fn get_left(&self, x: usize, y: usize) -> Option<(usize, usize)> {
    if x == 0 {
      None
    } else {
      Some((x - 1, y))
    }
  }
  
  fn get_right(&self, x: usize, y: usize) -> Option<(usize, usize)> {
    if x == self.get_width() - 1 {
      None
    } else {
      Some((x + 1, y))
    }
  }
  
  fn get_up(&self, x: usize, y: usize) -> Option<(usize, usize)> {
    if y == 0 {
      None
    } else {
      Some((x, y - 1))
    }
  }
  
  fn get_down(&self, x: usize, y: usize) -> Option<(usize, usize)> {
    if y == self.get_height() - 1 {
      None
    } else {
      Some((x, y + 1))
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
    let min = neighbors.iter().fold(u32::MAX,
      |acc, p| u32::min(acc, self.get_elevation(p.0, p.1)));
    self.get_elevation(x, y) < min
  }

  fn find_basins(&self) -> Vec<usize> {
    let mut seen: Vec<Vec<bool>> = Vec::new();
    // fill the basin array with zeros in the same shape
    for _ in 0..self.get_height() {
      seen.push(vec![false; self.get_width()])
    }

    // mark all of the max elevations as done
    for x in 0..self.get_width() {
      for y in 0..self.get_height() {
        if self.get_elevation(x, y) == MAP_RADIX - 1 {
          seen[y][x] = true;
        }
      }
    }

    let mut sizes: Vec<usize> = Vec::new();
    for x in 0..self.get_width() {
      for y in 0..self.get_height() {
        if !seen[y][x] {
          sizes.push(self.explore(x, y, &mut seen));
        }
      }
    }
    sizes
  }

  fn explore(&self, x: usize, y: usize, seen: &mut Vec<Vec<bool>>) -> usize {
    let mut to_do: Vec<(usize, usize)> = Vec::new();
    let mut size = 0;
    to_do.push((x,y));
    while to_do.len() > 0 {
      let (x, y) = to_do.pop().unwrap();
      if !seen[y][x] {
        seen[y][x] = true;
        size += 1;
        to_do.extend(self.get_left(x, y).into_iter());
        to_do.extend(self.get_right(x, y).into_iter());
        to_do.extend(self.get_up(x, y).into_iter());
        to_do.extend(self.get_down(x, y).into_iter());
      }
    }
    size
  }
}
