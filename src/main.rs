use std::fmt;
use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let mut scan = Scan::parse(&mut stdin.lock().lines()
       .map(|x| String::from(x.unwrap().trim()))
       .filter(|x| x.len() > 0));
  println!("original points = {}", scan.count());
  scan.next();
  scan.next();
  println!("enhanced points = {}", scan.count());
}

#[derive(Debug, Default)]
struct Scan {
  algorithm: Vec<bool>,
  map: Vec<Vec<bool>>,
  background: bool,
  width: usize,
}

impl Scan {
  fn convert(input: &str) -> Vec<bool> {
    input.chars().map(|c| c == '#').collect()
  }
  
  fn parse(input: &mut dyn Iterator<Item = String>) -> Self {
    let mut result = Scan::default();
    result.background = false;
    result.algorithm = Scan::convert(&input.next().unwrap());
    
    for line in input {
      result.map.push(Scan::convert(&line))
    }
    result.width = result.map.iter().map(|v| v.len())
        .reduce(|a,b| usize::min(a, b)).unwrap();
    result
  }

  fn lookup(&self, x: i64, y: i64) -> bool {
    if x < 0 || y < 0 ||
       x >= self.width as i64 || y >= self.map.len() as i64 {
      self.background
    } else {
      self.map[y as usize][x as usize]
    }
  }

  fn next_point(&self, x: i64, y: i64) -> bool {
    let mut idx: usize = 0;
    for y_nbr in -1..=1 {
      for x_nbr in -1..=1 {
        idx *= 2;
        if self.lookup(x + x_nbr, y + y_nbr) {
          idx += 1;
        }
      }
    }
    self.algorithm[idx]
  }

  fn next(&mut self) {
    let mut new_map: Vec<Vec<bool>> = Vec::new();
    for y in -1 ..= self.map.len() as i64 {
      let mut row: Vec<bool> = Vec::new();
      for x in -1 ..= self.width as i64 {
        row.push(self.next_point(x, y));
      }
      new_map.push(row);
    }
    self.background = self.algorithm[if self.background { 511 } else { 0 }];
    self.map = new_map;
    self.width += 2;
  }

  fn count(&self) -> usize {
    let mut result: usize = 0;
    for row in &self.map {
      for p in row {
        if *p {
          result += 1;
        }
      }
    }
    result
  }
}

impl fmt::Display for Scan {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for row in &self.map {
      for pos in row {
        write!(f, "{} ", if *pos { "#" } else { "." }) ?
      }
      write!(f, "\n") ?
    }
    fmt::Result::Ok(())
  }
}