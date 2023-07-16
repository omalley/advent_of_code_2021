
fn cost(n: i32) -> i32 {
  (n + 1) * n / 2
}

fn total_cost(posns: &Vec<i32>, goal: i32) -> i32 {
  posns.iter().fold(0, |total, x| total + cost((x - goal).abs()))
}

pub fn generator(data: &str) -> Vec<i32> {
  let first = data.lines()
    .map(|x| x.trim())
    .filter(|x| x.len() > 0).next().unwrap();
  let mut result: Vec<i32> = first.split(",")
    .map(|x| x.trim().parse::<i32>().unwrap())
    .collect();
  result.sort();
  result
}

pub fn part1(crabs: &Vec<i32>) -> i32 {
  let median = crabs[crabs.len() / 2];
  crabs.iter().fold(0, |cost, x| cost + (x - median).abs())
}

pub fn part2(crabs: &Vec<i32>) -> i32 {
  let mut best_cost = i32::MAX;
  for g in crabs[0]..crabs[crabs.len() - 1] {
    let new_cost = total_cost(&crabs, g);
    if new_cost < best_cost {
      best_cost = new_cost;
    }
  }
  best_cost
}

