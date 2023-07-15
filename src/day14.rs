use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Problem {
  initial: String,
  insertions: HashMap<String, Vec<String>>,
  current: HashMap<String, u64>,
}

impl Problem {
  fn parse_template(template: &str) -> HashMap<String, u64> {
    let mut result = HashMap::new();
    if template.len() > 1 {
      let mut char_itr = template.chars();
      let mut prev = char_itr.next().unwrap();
      for ch in char_itr {
        let mut key = String::from(prev);
        key.push(ch);
        result.insert(key.clone(), result.get(&key).unwrap_or(&0) + 1);
        prev = ch;
      }
    }
    result
  }
  
  fn parse(input: &mut dyn Iterator<Item = &str>) -> Self {
    let mut result = Problem::default();
    result.initial = input.next().unwrap().to_string();
    result.current = Problem::parse_template(&result.initial);
    for line in input {
      let parts: Vec<&str> = line.split("->").map(|s| s.trim()).collect();
      let key = String::from(parts[0]);
      let mut chars = key.chars();
      let mut val1 = String::from(chars.next().unwrap());
      val1.push_str(parts[1]);
      let mut val2 = String::from(parts[1]);
      val2.push(chars.next().unwrap());
      result.insertions.insert(key, vec![val1, val2]);
    }
    result
  }

  fn grow(&mut self) {
    let mut new_map : HashMap<String, u64> = HashMap::new();
    for (key, value) in &self.current {
      if self.insertions.contains_key(key) {
        for new_key in self.insertions.get(key).unwrap() {
          new_map.insert(String::from(new_key),
             new_map.get(new_key).unwrap_or(&0) + value);
        }
      } else {
        new_map.insert(String::from(key),
             new_map.get(key).unwrap_or(&0) + value);
      }
    }
    self.current = new_map;
  }

  fn score(&self) -> u64 {
    let mut char_cnt: HashMap<char, u64> = HashMap::new();
    // count the first character
    char_cnt.insert(self.initial.chars().next().unwrap(), 1);
    for (key, value) in &self.current {
      let ch = key.chars().last().unwrap();
      char_cnt.insert(ch, char_cnt.get(&ch).unwrap_or(&0) + *value);
    }
    let mut sum: Vec<u64> = char_cnt.iter().map(|(_,v)| *v).collect();
    sum.sort();
    sum.get(sum.len() - 1).unwrap() - sum.get(0).unwrap()
  }
}

pub fn generator(data: &str) -> Problem {
  Problem::parse(&mut data.lines()
    .map(|x| x.trim())
    .filter(|x| x.len() > 0))
}

pub fn part1(input: &Problem) -> u64 {
  let mut problem = (*input).clone();
  for _ in 0..10 {
    problem.grow();
  }
  problem.score()
}

pub fn part2(input: &Problem) -> u64 {
  let mut problem = (*input).clone();
  for _ in 0..40 {
    problem.grow();
  }
  problem.score()
}
