use std::cmp;

#[derive(Clone,Debug)]
pub struct Point {
  x: i64,
  y: i64,
}

impl Point {
  fn parse(s: &str) -> Point {
    let mut parts = s.split(",").map(|x| x.trim().parse::<i64>().unwrap());
    Point{x: parts.next().unwrap(), y: parts.next().unwrap()}
  }
}

#[derive(Clone,Debug)]
pub struct Line {
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

  fn is_upward(&self) -> bool {
    if self.is_vertical() || self.is_horizontal() {
      false
    } else if self.p2.x > self.p1.x {
      self.p2.y < self.p1.y
    } else {
      self.p2.y > self.p1.y
    }
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
      Bounding::Box{l, r: _, t, b: _} =>
        self.count[(p.x - l) as usize][(p.y - t) as usize] += 1
    }
  }
  
  fn add(&mut self, l: &Line) {
    if l.is_horizontal() {
      for x in l.left()..l.right()+1 {
        self.increment(&Point{x: x, y: l.top()})
      }
    } else if l.is_vertical() {
      for y in l.top()..l.bottom()+1 {
        self.increment(&Point{x: l.left(), y: y})
      }
    } else if l.is_upward() {
      for d in 0..(l.right() - l.left() + 1) {
        self.increment(&Point{x: l.left() + d,
                              y: l.bottom() - d});
      }
    } else {
      for d in 0..(l.right() - l.left() + 1) {
        self.increment(&Point{x: l.left() + d,
                              y: l.top() + d});
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

pub fn generator(data: &str) -> Vec<Line> {
  data.lines()
    .map(|x| x.trim())
    .filter(|x| x.len() > 0)
    .map(|x| Line::parse(x))
    .collect()
}

pub fn part1(lines: &Vec<Line>) -> i64 {
  let horiz_or_vert: Vec<Line> = lines.iter()
    .filter(|x| x.is_horizontal() || x.is_vertical())
    .cloned()
    .collect();
  let pic = Picture::new(&horiz_or_vert);
  pic.count(|x| x > 1)
}

pub fn part2(lines: &Vec<Line>) -> i64 {
  let pic = Picture::new(lines);
  pic.count(|x| x > 1)
}
