use std::cell::RefCell;
use std::fmt;
use std::io;
use std::io::BufRead;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::Chars;

fn main() {
  let stdin = io::stdin();
  let mut sum = SnailNumber::Number(0);
  let mut is_first = true;
  for line in stdin.lock().lines()
                   .map(|x| String::from(x.unwrap().trim()))
                   .filter(|x| x.len() > 0) {
    let mut number = SnailNumber::parse(&line);
    sum = if is_first {
      is_first = false;
      number
    } else {
      sum.add(&mut number)
    };
    println!("s = {}, mag = {}", sum, sum.magnitude());
  }
}

#[derive(Clone, Debug)]
enum SnailNumber {
  Number(i64),
  Pair(Rc<RefCell<SnailNumber>>, Rc<RefCell<SnailNumber>>),
}


enum ExplodeResult {
  None,
  Done,
  AddLeft(i64),
  AddRight(i64),
  AddBoth(i64,i64),
}

impl ExplodeResult {
  fn is_found(&self) -> bool {
    match self {
      ExplodeResult::None => false,
      _ => true,
    }
  }
}

impl SnailNumber {
  fn parse(input: &str) -> Self {
    SnailNumber::parse_item(&mut input.chars().peekable())
  }

  fn parse_item(input: &mut Peekable<Chars>) -> Self {
    match input.peek() {
      Some('[') => {
        input.next();
        let left = SnailNumber::parse_item(input);
        if input.next() != Some(',') {
          panic!("Missing ,");
        }
        let right = SnailNumber::parse_item(input);
        if input.next() != Some(']') {
          panic!("Missing ]");
        }
        SnailNumber::Pair(Rc::new(RefCell::new(left)), Rc::new(RefCell::new(right)))
      }
      Some('0'..='9') => SnailNumber::parse_number(input),
      Some(_) => panic!("Syntax error"),
      None => panic!("Empty stream"),
    }
  }

  fn parse_number(input: &mut Peekable<Chars>) -> Self {
    let mut s = String::new();
    while let Some(c) = input.next_if(|ch| ch.is_ascii_digit()) {
      s.push(c);
    }
    SnailNumber::Number(s.as_str().parse::<i64>().unwrap())
  }

  fn add(&mut self, right: &mut Self) -> Self {
    let mut result =
      SnailNumber::Pair(Rc::new(RefCell::new(self.clone())), Rc::new(RefCell::new(right.clone())));
    while result.explode(0).is_found() || result.split() {
      // pass
    }
    result
  }

  fn get_number(&self) -> i64 {
    match self {
      SnailNumber::Number(x) => *x,
      _ => 0,
    }
  }

  fn explode(&mut self, level: u64) -> ExplodeResult {
    match self {
      SnailNumber::Number(_) => ExplodeResult::None,
      SnailNumber::Pair(l, r) => {
        if level == 4 {
          let result = ExplodeResult::AddBoth(l.borrow().get_number(), r.borrow().get_number());
          *self = SnailNumber::Number(0);
          result
        } else {
          let result = l.borrow_mut().explode(level + 1);
          match result {
            ExplodeResult::AddRight(r_val) => {
              r.borrow_mut().add_to_leftmost(r_val);
              ExplodeResult::Done
            }
            ExplodeResult::AddBoth(l_val,r_val) => {
              r.borrow_mut().add_to_leftmost(r_val);
              ExplodeResult::AddLeft(l_val)
            }
            ExplodeResult::None => {
              let result = r.borrow_mut().explode(level + 1);
              match result {
                ExplodeResult::AddLeft(l_val) => {
                  l.borrow_mut().add_to_rightmost(l_val);
                  ExplodeResult::Done
                }
                ExplodeResult::AddBoth(l_val, r_val) => {
                  l.borrow_mut().add_to_rightmost(l_val);
                  ExplodeResult::AddRight(r_val)
                }
                _ => result,
              }
            }
            _ => result,
          }
        }
      }
    }
  }

  fn add_to_rightmost(&mut self, val: i64) {
    match self {
      SnailNumber::Number(n) => *n += val,
      SnailNumber::Pair(_, r) => r.borrow_mut().add_to_rightmost(val),
    }
  }

  fn add_to_leftmost(&mut self, val: i64) {
    match self {
      SnailNumber::Number(n) => *n += val,
      SnailNumber::Pair(l, _) => l.borrow_mut().add_to_leftmost(val),
    }
  }

  fn split(&mut self) -> bool {
    match self {
      SnailNumber::Number(n) => if *n < 10 { 
        false
      } else {
        *self = SnailNumber::Pair(Rc::new(RefCell::new(SnailNumber::Number(*n/2))),
                                  Rc::new(RefCell::new(SnailNumber::Number(*n - *n/2))));
        true
      }
      SnailNumber::Pair(l, r) => SnailNumber::split(&mut l.borrow_mut()) ||
                                 SnailNumber::split(&mut r.borrow_mut()),
    }
  }

  fn magnitude(&self) -> i64 {
   match self {
     SnailNumber::Number(n) => *n,
     SnailNumber::Pair(l, r) => 3 * l.borrow().magnitude() + 2 * r.borrow().magnitude(),
   }
  }
}

impl fmt::Display for SnailNumber {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      SnailNumber::Number(i) => write!(f, "{}", i),
      SnailNumber::Pair(left, right) => write!(f, "[{}, {}]", left.borrow(), right.borrow()),
    }
  }
}