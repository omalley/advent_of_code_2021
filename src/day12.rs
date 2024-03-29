use std::collections::HashMap;

#[derive(Default,Debug)]
pub struct CaveSystem {
  caves: HashMap<String,Cave>,
}

#[derive(Default,Debug)]
pub struct Cave {
  passages: Vec<String>,
  is_big: bool,
  is_end: bool,
}

impl CaveSystem {
  const START: &'static str = "start";
  const END: &'static str = "end";
  
  fn parse(input: &mut dyn Iterator<Item = &str>) -> Self {
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
    // Prevent links back to start or links out of end.
    if dest != CaveSystem::START && name != CaveSystem::END {
      self.caves.get_mut(name).unwrap().passages.push(String::from(dest));
    }
  }
}

#[derive(Debug)]
struct Decision {
  name: String,
  next: usize,
  used_double: bool,
}

impl Decision {
  fn new(name: &str, used_double: bool) -> Self {
    Decision{name: String::from(name), next: 0, used_double: used_double}
  }
}

#[derive(Debug)]
struct PathState<'a> {
  caves: &'a CaveSystem,
  path: Vec<Decision>,
}

impl<'a> PathState<'a> {
  fn new(caves: &'a CaveSystem, allow_double: bool) -> Self {
    PathState{path: vec![Decision::new(CaveSystem::START, !allow_double)],
              caves: caves.clone()}
  }

  // Is this a second visit to a small cave?
  fn is_double_visit(&self, next: &str, is_big: bool) -> bool {
    !is_big && self.path.iter().any(|x| x.name == next)
  }
}

impl<'a> Iterator for PathState<'a> {
  type Item = Vec<String>;
  
  fn next(&mut self) -> Option<Self::Item> {
    while self.path.len() > 0 {
      let last_entry: usize = self.path.len() - 1;
      let mut current = &mut self.path[last_entry];
      let used_double = current.used_double;
      let current_cave = &self.caves.caves[&current.name];
      if current.next >= current_cave.passages.len() {
        self.path.pop();
      } else {
        let next = &current_cave.passages[current.next];
        current.next += 1;
        let next_is_double = self.is_double_visit(next,
          self.caves.caves[next].is_big);

        if !used_double || !next_is_double {
          self.path.push(Decision::new(next, used_double || next_is_double));
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

pub fn generator(data: &str) -> CaveSystem {
  CaveSystem::parse(&mut data.lines()
    .map(|x| x.trim())
    .filter(|x| x.len() > 0))
}

pub fn part1(input: &CaveSystem) -> usize {
  let result = PathState::new(input, false);
  result.count()
}

pub fn part2(input: &CaveSystem) -> usize {
  let result = PathState::new(input, true);
  result.count()
}

