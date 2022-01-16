use std::collections::HashMap;
use std::io;
use std::io::BufRead;
use std::rc::Rc;

fn main() {
  let stdin = io::stdin();
  let caves = Rc::new(CaveSystem::parse(&mut stdin.lock().lines()
                        .map(|x| String::from(x.unwrap().trim()))
                        .filter(|x| x.len() > 0)));

  println!("paths = {}", PathState::new(caves.clone()).count());
}

#[derive(Default,Debug)]
struct CaveSystem {
  caves: HashMap<String,Cave>,
}

#[derive(Default,Debug)]
struct Cave {
  passages: Vec<String>,
  is_big: bool,
  is_end: bool,
}

impl CaveSystem {
  const START: &'static str = "start";
  const END: &'static str = "end";
  
  fn parse(input: &mut dyn Iterator<Item = String>) -> Self {
    let mut result = CaveSystem::default();
    for line in input {
      let parts: Vec<String> = line.split("-")
          .map(|x| String::from(x.trim())).collect();
      assert!(parts.len() == 2);
      result.create_cave(&parts[0], &parts[1]);
      result.create_cave(&parts[1], &parts[0]);
    }
    result
  }

  fn create_cave(&mut self, name: &str, dest: &str)  {
    if !self.caves.contains_key(name) {
      let mut new_cave = Cave::default();
      new_cave.is_big = name.chars().next().unwrap().is_ascii_uppercase();
      new_cave.is_end = name == CaveSystem::END;
      self.caves.insert(String::from(name), new_cave);
    }
    if name != CaveSystem::END {
      self.caves.get_mut(name).unwrap().passages.push(String::from(dest));
    }
  }
}

#[derive(Debug)]
struct Decision {
  name: String,
  next: usize,
}

impl Decision {
  fn new(name: &str) -> Self {
    Decision{name: String::from(name), next: 0}
  }
}

#[derive(Debug)]
struct PathState {
  caves: Rc<CaveSystem>,
  path: Vec<Decision>,
}

impl PathState {
  fn new(caves: Rc<CaveSystem>) -> Self {
    PathState{path: vec![Decision::new(CaveSystem::START)],
              caves: caves.clone()}
  }

  fn can_advance(&self, next: &str, is_big: bool) -> bool {
    is_big || !self.path.iter().any(|x| x.name == next)
  }
}

impl Iterator for PathState {
  type Item = Vec<String>;
  
  fn next(&mut self) -> Option<Self::Item> {
    while self.path.len() > 0 {
      let last_entry: usize = self.path.len() - 1;
      let mut current = &mut self.path[last_entry];
      let current_cave = &self.caves.caves[&current.name];
      if current.next >= current_cave.passages.len() {
        self.path.pop();
      } else {
        let next = &current_cave.passages[current.next];
        current.next += 1;
        if self.can_advance(next, self.caves.caves[next].is_big) {
          self.path.push(Decision::new(next));
        }
        if next == CaveSystem::END {
          return Some(self.path.iter()
                          .map(|x| x.name.clone())
                          .collect())
        }
      }
    }
    None
  }
}