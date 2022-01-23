use std::cmp::Ordering;
use std::io;
use std::io::BufRead;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

fn main() {
  let stdin = io::stdin();
  let scanners: Vec<Scanner> = Scanner::parse(&mut stdin.lock().lines()
       .map(|x| String::from(x.unwrap().trim()))
       .filter(|x| x.len() > 0));

  let mut solution = Solution::default();
  while solution.merged_scanners.len() < scanners.len() {
    println!("Merged: {:?}", solution.merged_scanners);
    let mut found = false;
    for scan in &scanners {
      if !solution.merged_scanners.contains(&scan.id) {
        if solution.merge(&scan) {
          found = true;
          break;
        }
      }
    }
    if !found {
      println!("Failed to find more matches");
      break;
    }
  }
  println!("sol = {:?}", solution);
  println!("beacons = {}", solution.beacons.len());
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct Point {
  x: i64,
  y: i64,
  z: i64,
}

impl Point {
  fn parse(input: &str) -> Self {
    let vals: Vec<i64> = input.split(",")
      .map(|x| x.trim().parse::<i64>().unwrap())
      .collect();
    Point{x: vals[0], y: vals[1], z: vals[2]}
  }

  fn add(&self, other: &Point) -> Point {
    Point{x: self.x + other.x, y: self.y + other.y, z: self.z + other.z}
  }

  fn subtract(&self, other: &Point) -> Point {
    Point{x: self.x - other.x, y: self.y - other.y, z: self.z - other.z}
  }
}

#[derive(Clone, Debug, Default)]
struct Scanner {
  id: i64,
  beacons: Vec<Point>,
}


impl Scanner {
  fn parse(input: &mut dyn Iterator<Item = String>) -> Vec<Self> {
    let mut result: Vec<Self> = Vec::new();
    let mut current = Scanner::default();
    for line in input {
      if line.starts_with("---") {
        if current.beacons.len() > 0 {
          result.push(current);
        }
        let words: Vec<&str>  = line.split_ascii_whitespace().collect();
        current = Scanner{id: words[2].parse::<i64>().unwrap(),
                          beacons: Vec::new()};
      } else {
        current.beacons.push(Point::parse(&line));
      }
    }
    if current.beacons.len() > 0 {
      result.push(current);
    }
    result
  }
}

#[derive(Debug, Default)]
struct Solution {
  beacons: Vec<Point>,
  merged_scanners: Vec<i64>,
}

impl Solution {
  const REQUIRED_MATCHES: usize = 12;
  
  fn merge(&mut self, scanner: &Scanner) -> bool {
    // the first scanner merges automatically
    if self.beacons.len() == 0 {
      self.add_points(scanner.id, &scanner.beacons, &Point::default());
      return true
    } else {
      for orient in Orientation::iter() {
        let mut points = Vec::new();
        for p in &scanner.beacons {
          points.push(orient.rotate(&p));
        }
        points.sort();
        if let Some(offset) = self.find_match(&points) {
          println!("Matched scanner {} with {:?} and {:?}", scanner.id, orient, offset);
          self.add_points(scanner.id, &points, &offset);
          return true
        }
      }
    }
    false
  }

  // merges a scanner's points into the current solution
  fn add_points(&mut self,
                id: i64,
                new_points: &Vec<Point>,
                offset: &Point) {
    for new in new_points {
      self.beacons.push(new.add(offset));
    }
    self.beacons.sort();
    self.beacons.dedup();
    self.merged_scanners.push(id);
  }
  
  // Tries to find a match with the current known beacons.
  // Assumes both sets of points are sorted.
  // Returns the offset to adjust the new_scanner points by
  fn find_match(&self, new_scanner: &Vec<Point>) -> Option<Point> {
    for old in 0 .. self.beacons.len() - Solution::REQUIRED_MATCHES {
      for new in 0 .. new_scanner.len() - Solution::REQUIRED_MATCHES {
        let offset = self.beacons[old].subtract(&new_scanner[new]);
        let mut matches: usize = 0;
        let mut left_posn: usize = old;
        let mut right_posn: usize = new;
        while left_posn < self.beacons.len() &&
              right_posn < new_scanner.len() &&
              new_scanner.len() - right_posn >=
                  Solution::REQUIRED_MATCHES - matches {
          let moved = new_scanner[right_posn].add(&offset);
          match self.beacons[left_posn].cmp(&moved) {
            Ordering::Less => left_posn += 1,
            Ordering::Greater => right_posn += 1,
            Ordering::Equal => {
              left_posn += 1;
              right_posn += 1;
              matches += 1;
              if matches == Solution::REQUIRED_MATCHES {
                return Some(offset);
              }
            }
          }
        }
      }
    }
    None
  }
}

#[derive(Debug, EnumIter)]
enum Orientation {
  ZposYpos,
  ZposXpos,
  ZposYneg,
  ZposXneg,
  YposXpos,
  YposZneg,
  YposXneg,
  YposZpos,
  XposYpos,
  XposZneg,
  XposYneg,
  XposZpos,
  ZnegYpos,
  ZnegXpos,
  ZnegYneg,
  ZnegXneg,
  YnegXpos,
  YnegZneg,
  YnegXneg,
  YnegZpos,
  XnegYpos,
  XnegZneg,
  XnegYneg,
  XnegZpos,
}

impl Orientation {
  fn rotate(&self, p: &Point) -> Point {
    match self {
      Orientation::XposYpos => Point{x: p.x, y: p.y, z: p.z},
      Orientation::XposZneg => Point{x: p.x, y: p.z, z: -p.y},
      Orientation::XposYneg => Point{x: p.x, y: -p.y, z: -p.z},
      Orientation::XposZpos => Point{x: p.x, y: -p.z, z: p.y},
      
      Orientation::ZposYpos => Point{x: -p.z, y: p.y, z: p.x},
      Orientation::ZposXpos => Point{x: p.y, y: p.z, z: p.x},
      Orientation::ZposYneg => Point{x: p.z, y: -p.y, z: p.x},
      Orientation::ZposXneg => Point{x: -p.y, y: -p.z, z: p.x},
      
      Orientation::YposXpos => Point{x: p.y, y: p.x, z: -p.z},
      Orientation::YposZneg => Point{x: -p.z, y: p.x, z: -p.y},
      Orientation::YposXneg => Point{x: -p.y, y: p.x, z: p.z},
      Orientation::YposZpos => Point{x: p.z, y: p.x, z: p.y},
      
      Orientation::ZnegYpos => Point{x: p.z, y: p.y, z: -p.x},
      Orientation::ZnegXpos => Point{x: p.y, y: -p.z, z: -p.x},
      Orientation::ZnegYneg => Point{x: -p.z, y: -p.y, z: -p.x},
      Orientation::ZnegXneg => Point{x: -p.y, y: p.z, z: -p.x},
      
      Orientation::YnegXpos => Point{x: p.y, y: -p.x, z: p.z},
      Orientation::YnegZneg => Point{x: p.z, y: -p.x, z: -p.y},
      Orientation::YnegXneg => Point{x: -p.y, y: -p.x, z: -p.z},
      Orientation::YnegZpos => Point{x: -p.z, y: -p.x, z: p.y},
      
      Orientation::XnegYpos => Point{x: -p.x, y: p.y, z: -p.z},
      Orientation::XnegZneg => Point{x: -p.x, y: -p.z, z: -p.y},
      Orientation::XnegYneg => Point{x: -p.x, y: -p.y, z: p.z},
      Orientation::XnegZpos => Point{x: -p.x, y: p.z, z: p.y},
    }
  }
}
