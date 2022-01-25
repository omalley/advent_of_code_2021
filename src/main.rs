use std::io;
use std::io::BufRead;

use lazy_static::lazy_static;
use regex::Captures;
use regex::Regex;

fn main() {
  let stdin = io::stdin();
  let cmds: Vec<Command> = stdin.lock().lines()
       .map(|x| String::from(x.unwrap().trim()))
       .filter(|x| x.len() > 0)
       .map(|x| Command::parse(&x))
       .collect();
  let mut reactor = Reactor::default();
  reactor.init(&cmds);
  println!("reactor = {} x {} x {}", reactor.x_cuts.len(),
      reactor.y_cuts.len(), reactor.z_cuts.len());
  for c in &cmds {
    reactor.run(c);
  }
  println!("count = {}", reactor.count());
}

#[derive(Debug, Default)]
struct Reactor {
  x_cuts: Vec<i64>,
  y_cuts: Vec<i64>,
  z_cuts: Vec<i64>,
  is_on: Vec<Vec<Vec<bool>>>,
}

impl Reactor {
  fn init(&mut self, cmds: &Vec<Command>) {
    for c in cmds {
      self.x_cuts.push(c.x0);
      self.x_cuts.push(c.x1+1);
      self.y_cuts.push(c.y0);
      self.y_cuts.push(c.y1+1);
      self.z_cuts.push(c.z0);
      self.z_cuts.push(c.z1+1);
    }
    self.x_cuts.sort();
    self.y_cuts.sort();
    self.z_cuts.sort();
    self.x_cuts.dedup();
    self.y_cuts.dedup();
    self.z_cuts.dedup();
    for _x in 0..self.x_cuts.len() {
      let mut row: Vec<Vec<bool>> = Vec::new();
      for _y in 0..self.y_cuts.len() {
        row.push(vec![false; self.z_cuts.len()]);
      }
      self.is_on.push(row);
    }
  }

  fn run(&mut self, cmd: &Command) {
    for x in self.x_idx(cmd.x0)..self.x_idx(cmd.x1+1) {
      for y in self.y_idx(cmd.y0)..self.y_idx(cmd.y1+1) {
        for z in self.z_idx(cmd.z0)..self.z_idx(cmd.z1+1) {
          self.is_on[x][y][z] = cmd.on;
        }
      }
    }
  }

  fn x_idx(&self, x: i64) -> usize {
    self.x_cuts.binary_search(&x).unwrap()
  }

  fn y_idx(&self, y: i64) -> usize {
    self.y_cuts.binary_search(&y).unwrap()
  }

  fn z_idx(&self, z: i64) -> usize {
    self.z_cuts.binary_search(&z).unwrap()
  }

  fn count(&self) -> usize {
    let mut result: usize = 0;
    for x in 0..self.x_cuts.len() - 1 {
      for y in 0..self.y_cuts.len() - 1 {
        for z in 0..self.z_cuts.len() - 1 {
          if self.is_on[x][y][z] {
            result += (self.x_cuts[x+1] - self.x_cuts[x]) as usize *
                (self.y_cuts[y+1] - self.y_cuts[y]) as usize*
                (self.z_cuts[z+1] - self.z_cuts[z]) as usize;
          }
        }
      }
    }
    result
  }
}

#[derive(Debug, Default)]
struct Command {
  on: bool,
  x0: i64,
  x1: i64,
  y0: i64,
  y1: i64,
  z0: i64,
  z1: i64,
}

impl Command {
  const MIN: i64 = -50;
  const MAX: i64 = 50;
  fn parse(input: &str) -> Self {
    lazy_static! {
      static ref LINE_RE: Regex = Regex::new("^(?P<cmd>on|off)\\s+\
          x=(?P<x0>-?\\d+)..(?P<x1>-?\\d+),\
          y=(?P<y0>-?\\d+)..(?P<y1>-?\\d+),\
          z=(?P<z0>-?\\d+)..(?P<z1>-?\\d+)$").unwrap();
    }
    let capture = LINE_RE.captures(input).unwrap();
    Command{on: capture.name("cmd").unwrap().as_str() == "on",
            x0: i64::max(Command::MIN, number(&capture, "x0")),
            x1: i64::min(Command::MAX, number(&capture, "x1")),
            y0: i64::max(Command::MIN, number(&capture, "y0")),
            y1: i64::min(Command::MAX, number(&capture, "y1")),
            z0: i64::max(Command::MIN, number(&capture, "z0")),
            z1: i64::min(Command::MAX, number(&capture, "z1"))}
  }
}

fn number(capture: &Captures, name: &str) -> i64 {
  capture.name(name).unwrap().as_str().parse::<i64>().unwrap()
}

