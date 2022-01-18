use std::io;
use std::io::BufRead;

use lazy_static::lazy_static;
use regex::Captures;
use regex::Regex;

fn main() {
  let stdin = io::stdin();
  for line in stdin.lock().lines()
                   .map(|x| String::from(x.unwrap().trim()))
                   .filter(|x| x.len() > 0) {
    let target = Target::parse(&line);
    let mut best = (0, 0, i64::MIN);
    let mut count: i64 = 0;
    for x_speed in 0..178 {
      for y_speed in -2000..3000 {
        if let Some(height) = target.is_hit(x_speed, y_speed) {
          count += 1;
          if height > best.2 {
            best = (x_speed, y_speed, height);
          }
        }
      }
    }
    println!("best = {:?}, count = {}", best, count);
  }
}

#[derive(Debug)]
struct Target {
  left: i64,
  right: i64,
  bottom: i64,
  top: i64,
}

impl Target {
  fn get_number(cap: &Captures, name: &str) -> i64 {
    cap.name(name).unwrap().as_str().parse::<i64>().unwrap()
  }

  fn parse(input: &str) -> Self {
    lazy_static! {
      static ref BOX_RE: Regex =
        Regex::new(concat!(r"^target area: x=(?P<left>-?\d+)\.\.",
                           r"(?P<right>-?\d+),",
                           r"\s+y=(?P<bottom>-?\d+)..",
                           r"(?P<top>-?\d+)$")).unwrap();
    }
    let cap = BOX_RE.captures(input).unwrap();
    Target{left: Target::get_number(&cap, "left"),
           right: Target::get_number(&cap, "right"),
           bottom: Target::get_number(&cap, "bottom"),
           top: Target::get_number(&cap, "top")}
  }

  fn is_hit(&self, x_speed: i64, y_speed: i64) -> Option<i64> {
    let mut x: i64 = 0;
    let mut y: i64 = 0;
    let mut x_delta: i64 = x_speed;
    let mut y_delta: i64 = y_speed;
    let mut max_y: i64 = self.bottom;
    while (x_delta != 0 || (x >= self.left && x <= self.right)) &&
          (y >= self.bottom || y_delta > 0) {
      x += x_delta;
      y += y_delta;
      max_y = i64::max(max_y, y);
      if x_delta > 0 {
        x_delta -= 1;
      } else if x_delta < 0 {
        x_delta += 1;
      }
      y_delta -= 1;
      if x >= self.left && x <= self.right &&
         y >= self.bottom && y <= self.top {
        return Some(max_y)
      }
    }
    None
  }
}  