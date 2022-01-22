use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let scanners: Vec<Scanner> = Scanner::parse(&mut stdin.lock().lines()
       .map(|x| String::from(x.unwrap().trim()))
       .filter(|x| x.len() > 0));
  for s in scanners {
    println!("s = {:?}", s);
  }
}

#[derive(Clone, Copy, Debug, Default)]
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