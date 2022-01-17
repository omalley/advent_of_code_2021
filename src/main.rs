use std::collections::HashMap;
use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let mut problem = Problem::parse(&mut stdin.lock().lines()
                                     .map(|x| String::from(x.unwrap().trim()))
                                     .filter(|x| x.len() > 0));
  const ITERATIONS: i64 = 10;
  for i in 0..ITERATIONS {
    println!("iteration {}", i);
    problem.grow();
  }
  println!("iter {} = {}", ITERATIONS, problem.score());
}

#[derive(Debug, Default)]
struct Problem {
  current: String,
  insertions: HashMap<String, String>,
}

impl Problem {
  fn parse(input: &mut dyn Iterator<Item = String>) -> Self {
    let mut result = Problem::default();
    result.current = input.next().unwrap();
    for line in input {
      let parts: Vec<&str> = line.split("->").map(|s| s.trim()).collect();
      result.insertions.insert(String::from(parts[0]), String::from(parts[1]));
    }
    result
  }

  fn grow(&mut self) {
    if self.current.len() > 1 {
      let mut chars = self.current.chars();
      let mut result = String::new();
      let mut prev = chars.next().unwrap();
      result.push(prev);
      for cur in chars {
        let mut key = String::from(prev);
        key.push(cur);
        if self.insertions.contains_key(&key) {
          result.push_str(self.insertions.get(&key).unwrap());
        }
        result.push(cur);
        prev = cur;
      }
      self.current = result;
    }
  }

  fn score(&self) -> u64 {
    let mut cnt: HashMap<char, u64> = HashMap::new();
    for ch in self.current.chars() {
      cnt.insert(ch, cnt.get(&ch).unwrap_or(&0) + 1);
    }
    let mut sum: Vec<u64> = cnt.iter().map(|(_c,v)| *v).collect();
    sum.sort();
    sum.get(sum.len() - 1).unwrap() - sum.get(0).unwrap()
  }
}