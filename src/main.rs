use std::io;
use std::io::BufRead;

fn main() {
  // Get the input lines
  let lines: Vec<String> = io::stdin().lock().lines()
    .map(|x| String::from(x.unwrap().trim())).collect();
  // Convert them to integers
  let nums: Vec<i32> = lines.iter().map(|x| x.parse::<i32>().unwrap()).collect();
  print!("Descended {}\n", count_descents(&nums));
  print!("Triple Descended {}\n", count_triple_descents(&nums));
}

fn count_descents(nums: &Vec<i32>) -> i32 {
  let mut count = 0;
  let mut last = i32::MAX;
  for current in nums {
    if *current > last {
      count += 1;
    }
    last = *current;
  }
  count
}

fn count_triple_descents(nums: &Vec<i32>) -> i32 {
  let mut count = 0;
  let mut last = i32::MAX;
  for i in 0..nums.len()-2 {
    let current = nums[i] + nums[i+1] + nums[i+2];
    if current > last {
      count += 1;
    }
    last = current;
  }
  count
}