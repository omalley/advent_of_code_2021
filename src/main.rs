use std::collections::HashMap;
use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let lines: Vec<Display> = stdin.lock().lines()
     .map(|x| String::from(x.unwrap().trim()))
     .filter(|x| x.len() > 0)
     .map(|x| Display::parse(&x))
     .collect();

  let easy = lines.iter().map(|x| x.easy_numbers())
    .fold(0, |acc, x| acc + x);
  let sum = lines.iter().map(|x| x.unscramble())
    .fold(0, |acc, x| acc + x);
  println!("easy = {}, sum = {}", easy, sum);
}

#[derive(Debug,Default)]
struct Display {
  digits: Vec<String>,
  display: Vec<String>,
}

impl Display {
  fn parse(input: &str) -> Self {
    let mut result = Display::default();
    let parts: Vec<String> = input.split("|")
        .map(|x| String::from(x)).collect();
    result.digits = parts[0].split_whitespace().
        map(|x| sort_word(x.trim())).collect();
    result.digits.sort_by(|a, b| a.len().partial_cmp(&b.len()).unwrap());
    result.display = parts[1].split_whitespace().
        map(|x| sort_word(x.trim())).collect();
    result
  }

  fn easy_numbers(&self) -> i32 {
    let easy = vec![2, 3, 4, 7];
    self.display.iter().filter(|x| easy.contains(&x.len())).count() as i32
  }

  fn unscramble(&self) -> i32 {
    let mut trans = HashMap::new();
    // map the easy ones
    trans.insert(&self.digits[0], 1);
    trans.insert(&self.digits[1], 7);
    trans.insert(&self.digits[2], 4);
    trans.insert(&self.digits[9], 8);

    // now look at the 5 segment ones
    for s in &self.digits[3..6] {
      if overlap(s, &self.digits[0]) == 2 {
        trans.insert(s, 3);
      } else if overlap(s, &self.digits[2]) == 2 {
        trans.insert(s, 2);
      } else {
        trans.insert(s, 5);
      }
    }

    // now look at the 6 segment ones
    for s in &self.digits[6..9] {
      if overlap(s, &self.digits[2]) == 4 {
        trans.insert(s, 9);
      } else if overlap(s, &self.digits[0]) == 2 {
        trans.insert(s, 0);
      } else {
        trans.insert(s, 6);
      }
    }
    self.display.iter().fold(0, |acc, x| acc * 10 + trans.get(x).unwrap())
  }
}

fn sort_word(input: &str) -> String {
  let mut chars: Vec<char> = input.chars().collect();
  chars.sort();
  String::from_iter(chars)
}

fn overlap(long: &str, short: &str) -> i32 {
  let goal: Vec<char> = short.chars().collect();
  long.chars().filter(|c| goal.contains(c)).count() as i32
}