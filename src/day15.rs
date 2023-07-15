use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Point {
  x: usize,
  y: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ToDoItem {
  cost: u32,
  position: Point,
}

impl Ord for ToDoItem {
  fn cmp(&self, other: &Self) -> Ordering {
    other.cost.cmp(&self.cost)
         .then_with(|| self.position.cmp(&other.position))
  }
}

impl PartialOrd for ToDoItem {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[derive(Debug)]
pub struct Problem {
  risk: Vec<Vec<u32>>,
  width: usize,
}

impl Problem {
  const RISK_RADIX: u32 = 10;
  
  fn parse(input: &mut dyn Iterator<Item = &str>) -> Self {
    let risk: Vec<Vec<u32>> = input
      .map(|l| l.chars()
        .map(|c| c.to_digit(Problem::RISK_RADIX).unwrap())
        .collect())
      .collect();
    let width = risk.iter().map(|r| r.len()).min().unwrap();
    Problem{risk, width}
  }

  fn find_neighbors(&self, point: &Point) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();
    if point.x > 0 {
      result.push(Point{x: point.x - 1, y: point.y});
    }
    if point.y > 0 {
      result.push(Point{x: point.x, y: point.y - 1});
    }
    if point.y < self.risk.len() - 1 {
      result.push(Point{x: point.x, y: point.y + 1});
    }
    if point.x < self.width - 1 {
      result.push(Point{x: point.x + 1, y: point.y});
    }
    result
  }
  
  fn find_lowest(&self) -> u32 {
    let mut best: Vec<Vec<u32>> =
        vec![vec![ u32::MAX; self.width]; self.risk.len()];
    best[0][0] = 0;
    let mut to_do: BinaryHeap<ToDoItem> = BinaryHeap::new();
    to_do.push(ToDoItem{cost:0, position: Point{x:0, y:0}});
    while let Some(ToDoItem{cost: _, position}) = to_do.pop() {
      for neighbor in &self.find_neighbors(&position) {
        let new_risk =
            self.risk[neighbor.y][neighbor.x] + best[position.y][position.x];
        if new_risk < best[neighbor.y][neighbor.x] {
          best[neighbor.y][neighbor.x] = new_risk;
          to_do.push(ToDoItem{cost: new_risk, position: neighbor.clone()});
        }
      }
    }
    best[self.risk.len() -1][self.width - 1]
  }

  /// Return a copy of self with the matrix replicated multiple times
  /// in each dimension.
  fn multiply(&self, multiple: usize) -> Self {
    let width = self.width * multiple;
    let mut risk = Vec::new();
    for y_tile in 0..multiple {
      for template_row in &self.risk {
        let mut new_row = Vec::new();
        for x_tile in 0..multiple {
          for val in template_row {
            new_row.push((val + (y_tile + x_tile) as u32 - 1) %
              (Problem::RISK_RADIX - 1) + 1);
          }
        }
        risk.push(new_row);
      }
    }
    Problem{risk, width}
  }
}

pub fn generator(data: &str) -> Problem {
  Problem::parse(&mut data.lines())
}

pub fn part1(problem: &Problem) -> u32 {
  problem.find_lowest()
}

pub fn part2(problem: &Problem) -> u32 {
  problem.multiply(5).find_lowest()
}
