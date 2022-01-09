use std::cmp;
use std::io;
use std::io::BufRead;
use std::iter;

fn main() {
  // Get the input lines
  let lines: Vec<String> = io::stdin().lock().lines()
    .map(|x| String::from(x.unwrap().trim())).collect();
  let trans = transpose(&lines);
  let gamma = compute_gamma(&trans);
  let epsilon = (1 << trans.len()) - gamma - 1;
  println!("gamma = {}, epsilon = {}, product = {}", gamma, epsilon,
           gamma * epsilon);
}

// Transpose the strings so that the first character of each row beomes
// the first row.
fn transpose(input: &Vec<String>) -> Vec<String> {
  let width =
    input.iter().map(|x| x.len()).reduce(|l, r| cmp::max(l,r)).unwrap();
  // start with width empty strings
  let mut result: Vec<String>
    = iter::repeat(String::new()).take(width).collect();
  // for each line, append each character
  for s in input {
    let mut i = 0;
    for ch in s.chars() {
      result[i].push(ch);
      i += 1;
    }
  }
  result
}

// compare the number of '1' versus '0' in the string
// returns the difference
fn compare_digits(s: &str) -> i64 {
  let mut result = 0;
  for c in s.chars() {
    match c {
      '0' => result -= 1,
      '1' => result += 1,
      _ => eprintln!("Unknown character {}", c),
    }
  }
  result
}

fn compute_gamma(s: &Vec<String>) -> i64 {
  let mut result = 0;
  for diff in s.iter().map(|x| compare_digits(x)) {
    result *= 2;
    if diff > 0 {
      result += 1;
    }
  }
  result
}