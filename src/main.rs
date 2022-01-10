use std::cmp;
use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let mut lines: Vec<Line> = stdin.lock().lines()
     .map(|x| String::from(x.unwrap().trim()))
     .filter(|x| x.len() > 0)
     .map(|x| Line::parse(x.as_str()))
     .collect();

  lines.retain(|x| x.is_vertical() || x.is_horizontal());
  let pic = Picture::new(&lines);
  
  println!("pic = {:?}", pic);
  println!("result = {}", pic.count(|x| x > 1));
}

#[derive(Debug)]
struct Point {
  x: i64,
  y: i64,
}

impl Point {
  fn parse(s: &str) -> Point {
    let mut parts = s.split(",").map(|x| x.trim().parse::<i64>().unwrap());
    Point{x: parts.next().unwrap(), y: parts.next().unwrap()}
  }
}

#[derive(Debug)]
struct Line {
  p1: Point,
  p2: Point,
}

impl Line {
  fn parse(s: &str) -> Line {
    let mut parts = s.split("->").map(|x| Point::parse(x.trim()));
    Line{p1: parts.next().unwrap(), p2: parts.next().unwrap()}
  }

  fn is_vertical(&self) -> bool {
    self.p1.x == self.p2.x
  }
  
  fn is_horizontal(&self) -> bool {
    self.p1.y == self.p2.y
  }

  fn left(&self) -> i64 {
    cmp::min(self.p1.x, self.p2.x)
  }

  fn right(&self) -> i64 {
    cmp::max(self.p1.x, self.p2.x)
  }

  fn top(&self) -> i64 {
    cmp::min(self.p1.y, self.p2.y)
  }

  fn bottom(&self) -> i64 {
    cmp::max(self.p1.y, self.p2.y)
  }
}

#[derive(Debug)]
enum Bounding {
  Empty,
  Box{l:i64, r:i64, t:i64, b:i64},
}

impl Default for Bounding {
  fn default() -> Self { Bounding::Empty }
}

impl Bounding {
  fn add(&self, line: &Line) -> Bounding {
    match self {
      Bounding::Empty => Bounding::Box{
        l: line.left(), r: line.right(),
        t: line.top(), b: line.bottom()},
      Bounding::Box{l, r, t, b} => Bounding::Box{
        l: cmp::min(*l, line.left()),
        r: cmp::max(*r, line.right()),
        t: cmp::min(*t, line.top()),
        b: cmp::max(*b, line.bottom())}
    }
  }
}

#[derive(Debug, Default)]
struct Picture {
  bounds: Bounding,
  count: Vec<Vec<i32>>,
}

impl Picture {
  fn new(lines: &Vec<Line>) -> Self {
    let mut result = Picture::default();
    result.bounds = lines.iter().fold(result.bounds, |b, l| b.add(l));
    match result.bounds {
      Bounding::Empty => {}
      Bounding::Box{l, r, t, b} =>
        for _x in l..r+1 {
          result.count.push(Vec::new());
          for _y in t..b+1 {
            result.count.last_mut().unwrap().push(0);
          }
        }
    }
    for l in lines {
      result.add(l);
    }
    result
  }

  fn increment(&mut self, p: &Point) {
    match self.bounds {
      Bounding::Empty => {},
      Bounding::Box{l, r, t, b} =>
        self.count[(p.x - l) as usize][(p.y - t) as usize] += 1
    }
  }
  
  fn add(&mut self, l: &Line) {
    if l.is_horizontal() {
      for x in l.left()..l.right()+1 {
        self.increment(&Point{x: x, y: l.top()})
      }
    } else {
      for y in l.top()..l.bottom()+1 {
        self.increment(&Point{x: l.left(), y: y})
      }
    }
  }

  fn count<F>(&self, f: F) -> i64
      where F: Fn(i32) -> bool {
    let mut result: i64 = 0;
    for col in &self.count {
      for loc in col {
        if f(*loc) {
          result += 1;
        }
      }
    }
    result
  }
}