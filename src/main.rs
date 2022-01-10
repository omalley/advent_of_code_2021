use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let mut lines = stdin.lock().lines()
     .map(|x| String::from(x.unwrap().trim()))
     .filter(|x| x.len() > 0);

  let mut ocean = Ocean::default();
  // parse the first line as fish ages
  for fish in lines.next().unwrap().split(",")
               .map(|x| x.trim().parse::<i32>().unwrap()) {
     ocean.add(fish, 1);
  }

  for _ in 0..80 {
    ocean.age();
    println!("ocean = {:?}", ocean);
  }
  println!("total = {}", ocean.total());  
}

const BIRTH_TO_BIRTH: i32 = 9;
const GENERATION: i32 = 7;
  
#[derive(Debug,Default)]
struct Ocean {
  count: Vec<i32>,
  age: i32,
}

impl Ocean {
  fn add(&mut self, age: i32, cnt: i32) {
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

  fn total(&self) -> i32 {
    self.count.iter().fold(0, |acc, c| acc + c)
  }
}