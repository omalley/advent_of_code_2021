use lazy_static::lazy_static;
use regex::Captures;
use regex::Regex;

#[derive(Clone,Debug)]
pub enum Fold {
  Horizontal {y : usize},
  Vertical {x: usize}
}

#[derive(Clone,Copy,Debug,Eq,Ord,PartialEq,PartialOrd)]
pub struct Point {
  x: usize,
  y: usize,
}

#[derive(Clone,Default,Debug)]
pub struct Problem {
  points: Vec<Point>,
  folds: Vec<Fold>,
}

impl Problem {
  fn get_number(cap: &Captures, name: &str) -> usize {
    cap.name(name).unwrap().as_str().parse::<usize>().unwrap()
  }
  
  fn parse(input: &mut dyn Iterator<Item = &str>) -> Self {
    lazy_static! {
      static ref POINT_RE: Regex
        = Regex::new(r"^(?P<x>\d+),(?P<y>\d+)$").unwrap();
      static ref FOLD_RE: Regex
        = Regex::new(r"^fold\s+along\s+(?P<dir>[xy])=(?P<val>\d+)$").unwrap();
    }
    let mut result = Problem::default();
    for line in input {
      match FOLD_RE.captures(&line) {
        None =>
          match POINT_RE.captures(&line) {
            None => panic!("Bad point {}", line),
            Some(cap) =>
              result.points.push(Point{
                x: Problem::get_number(&cap, "x"),
                y: Problem::get_number(&cap, "y")
              }),
          }
        Some(cap) =>
          match cap.name("dir").unwrap().as_str() {
            "x" => result.folds.push(Fold::Vertical{
              x: Problem::get_number(&cap, "val")}),
            "y" => result.folds.push(Fold::Horizontal{
              y: Problem::get_number(&cap, "val")}),
            _ => panic!("Bad fold {}", line),
          }
      }
    }
    result
  }

  fn do_fold(&mut self, fold_idx: usize) {
    match self.folds.get(fold_idx).unwrap() {
      Fold::Vertical{x: vf} =>
        self.points =
          self.points.iter().map(|p|
            if p.x > *vf {
              Point{x: 2* vf - p.x, y: p.y}
            } else {
              *p
            }).collect(),
      Fold::Horizontal{y: hf} =>
        self.points =
          self.points.iter().map(|p|
            if p.y > *hf {
              Point{x: p.x, y: 2 * hf - p.y}
            } else {
              *p
            }).collect(),
    }
    self.points.sort_unstable();
    self.points.dedup();
  }

  fn count(&self) -> usize {
    self.points.len()
  }

  fn draw(&self) -> String {
    let mut picture = String::new();
    let max_x = self.points.iter()
        .map(|p| p.x).fold(0, |a,b| usize::max(a, b));
    let max_y = self.points.iter()
        .map(|p| p.y).fold(0, |a,b| usize::max(a, b));
    let mut result: Vec<Vec<bool>> =
      vec![vec![false; max_x + 1]; max_y + 1];
    for p in &self.points {
      result[p.y][p.x] = true;
    }
    for line in result {
      for posn in line {
        picture.push_str(if posn {"#"} else {" "})
      }
      picture.push_str("\n");
    }
    picture
  }
}

pub fn generator(data: &str) -> Problem {
  Problem::parse(&mut data.lines()
    .map(|x| x.trim())
    .filter(|x| x.len() > 0))
}

pub fn part1(input: &Problem) -> String {
  let mut problem = (*input).clone();
  problem.do_fold(0);
  problem.count().to_string()
}

pub fn part2(input: &Problem) -> String {
  let mut problem = (*input).clone();
  for f in 0..problem.folds.len() {
    problem.do_fold(f);
  }
  problem.draw()
}

