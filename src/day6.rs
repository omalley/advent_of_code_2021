const BIRTH_TO_BIRTH: i32 = 9;
const GENERATION: i32 = 7;
  
#[derive(Clone,Debug,Default)]
pub struct Ocean {
  count: Vec<i64>,
  age: i32,
}

impl Ocean {
  fn add(&mut self, age: i32, cnt: i64) {
    while self.count.len() <= age as usize {
      self.count.push(0);
    }
    self.count[age as usize] += cnt;
  }

  fn age(&mut self) {
    let children = self.count.remove(0);
    self.add(BIRTH_TO_BIRTH - 1, children);
    self.add(GENERATION - 1, children);
    self.age += 1;
  }

  fn total(&self) -> i64 {
    self.count.iter().fold(0, |acc, c| acc + c)
  }
}

pub fn generator(data: &str) -> Ocean {
  let first = data.lines()
    .map(|x| x.trim())
    .filter(|x| x.len() > 0).next().unwrap();
  let mut ocean = Ocean::default();
  for fish in first.split(",") {
    let age = fish.trim().parse::<i32>().unwrap();
    ocean.add(age, 1);
  }
  ocean
}

pub fn part1(fishes: &Ocean) -> i64 {
  let mut ocean = (*fishes).clone();
  for _ in 0..80 {
    ocean.age();
  }
  ocean.total()
}

pub fn part2(fishes: &Ocean) -> i64 {
  let mut ocean = (*fishes).clone();
  for _ in 0..256 {
    ocean.age();
  }
  ocean.total()
}

