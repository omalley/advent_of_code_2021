use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let problem = Problem::parse(&mut stdin.lock().lines()
                                 .map(|x| String::from(x.unwrap().trim()))
                                 .filter(|x| x.len() > 0));
  println!("lowest = {}", problem.find_lowest());
}

#[derive(Debug, Default)]
struct Problem {
  risk: Vec<Vec<u32>>,
  width: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct Point {
  x: usize,
  y: usize,
}

impl Problem {
  const RISK_RADIX: u32 = 10;
  
  fn parse(input: &mut dyn Iterator<Item = String>) -> Self {
    let mut result = Problem::default();
    result.risk = input.map(|line| line.chars()
                             .map(|c| c.to_digit(Problem::RISK_RADIX).unwrap())
                             .collect())
                       .collect();
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
    let mut to_do: Vec<Point> = Vec::new();
    to_do.push(Point{x:0, y:0});
    while to_do.len() > 0 {
      let current = to_do.pop().unwrap();
      for neighbor in &self.find_neighbors(&current) {
        let new_risk =
            self.risk[neighbor.y][neighbor.x] + best[current.y][current.x];
        if new_risk < best[neighbor.y][neighbor.x] {
          best[neighbor.y][neighbor.x] = new_risk;
          to_do.push(neighbor.clone());
        }
      }
    }
    println!("best = {:?}", best);
    best[self.risk.len() -1][self.width - 1]
  }
}
