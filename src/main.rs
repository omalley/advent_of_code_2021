use std::io;
use std::io::BufRead;

fn main() {
  // Get the input lines
  let lines: Vec<String> = io::stdin().lock().lines()
    .map(|x| String::from(x.unwrap().trim())).collect();
  // Convert them to integers
  let nums: Vec<i32> = lines.iter().map(|x| x.parse::<i32>().unwrap()).collect();
  print!("Descended {}\n", count_descents(&nums));
}

fn count_descents(nums: &Vec<i32>) -> i32 {
  let mut result = 0;
  let mut last = i32::MAX;
  for current in nums {
    if *current > last {
      result += 1;
    }
    last = *current;
  }
  result
}