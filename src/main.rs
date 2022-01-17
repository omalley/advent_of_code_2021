use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let problem = Problem::parse(&mut stdin.lock().lines()
                                 .map(|x| String::from(x.unwrap().trim()))
                                 .filter(|x| x.len() > 0), 5);
//  println!("board = {:?}", problem);                                 
  println!("lowest = {}", problem.find_lowest());
}

#[derive(Debug, Default)]
struct Problem {
  risk: Vec<Vec<u32>>,
  width: usize,
}

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

impl Problem {
  const RISK_RADIX: u32 = 10;
  
  fn parse(input: &mut dyn Iterator<Item = String>,
           multiplier: u32) -> Self {
    let mut result = Problem::default();
    // read the template in
    let template: Vec<Vec<u32>> = input.map(|line| line.chars()
       .map(|c| c.to_digit(Problem::RISK_RADIX).unwrap()).collect())
       .collect();
    for tile_y in 0..multiplier {
      for template_row in &template {
        let mut row: Vec<u32> = Vec::new();
        for tile_x in 0..multiplier {
          for val in template_row {
            row.push((val + tile_y + tile_x - 1) %
                        (Problem::RISK_RADIX - 1) + 1);
          }
        }
        result.risk.push(row);
      }
    }
    result.width =
      match result.risk.iter().map(|x| x.len())
              .reduce(|a, b| usize::min(a, b)) {
        None => 0,
        Some(x) => x,
      };
    result
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
}
