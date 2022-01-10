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
  let median = crabs[crabs.len() / 2];
  let cost = crabs.iter().fold(0, |cost, x| cost + (x - median).abs());
  println!("location = {}, cost = {}", median, cost);  
}

