use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let mut lines = stdin.lock().lines()
     .map(|x| String::from(x.unwrap().trim()))
     .filter(|x| x.len() > 0);

  // parse the first line as crab locations
  let mut crabs: Vec<i32> = lines.next().unwrap().split(",")
      .map(|x| x.trim().parse::<i32>().unwrap()).collect();

  // sort them and find median
  crabs.sort();
  let mut best_cost = i32::MAX;
  let mut best = -1;
  for g in crabs[0]..crabs[crabs.len() - 1] {
    let new_cost = total_cost(&crabs, g);
    if new_cost < best_cost {
      best_cost = new_cost;
      best = g;
    }
  }
  println!("location = {}, cost = {}", best, best_cost);  
}

fn cost(n: i32) -> i32 {
  (n + 1) * n / 2
}

fn total_cost(posns: &Vec<i32>, goal: i32) -> i32 {
  posns.iter().fold(0, |total, x| total + cost((x - goal).abs()))
}