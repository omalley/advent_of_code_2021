#[derive(Debug)]
pub enum ParseResult {
  OK,
  Corrupted{_expect: char, found: char},
  Incomplete{expect: Vec<char>},
  Illegal(char),
  Underflow,
}

fn score(close: char) -> u64 {
  match close {
    ')' => 3,
    ']' => 57,
    '}' => 1197,
    '>' => 25137,
    _ => 0,
  }
}

fn fix_score(close: &Vec<char>) -> u64 {
  close.iter()
    .map(|c|
      match c {
        ')' => 1,
        ']' => 2,
        '}' => 3,
        '>' => 4,
        _ => 0})
     .fold(0, |a, b| 5 * a + b)
}

fn closer(start: char) -> Option<char> {
  match start {
    '[' => Some(']'),
    '(' => Some(')'),
    '<' => Some('>'),
    '{' => Some('}'),
    _ => None,
  }
}

fn is_close(close: char) -> bool {
  match close {
    ']' | ')' | '>' | '}' => true,
    _ => false,
  }
}

fn parse(input: &str) -> ParseResult {
  let mut stack: Vec<char> = Vec::new();
  for ch in input.chars() {
    if is_close(ch) {
      let top = stack.pop();
      match top {
        None => return ParseResult::Underflow,
        Some(req) => if req != ch {
          return ParseResult::Corrupted{_expect: req, found: ch}
        }
      }
    } else {
      let close = closer(ch);
      match close {
        None => return ParseResult::Illegal(ch),
        Some(goal) => stack.push(goal),
      }
    }
  }
  if stack.len() == 0 {
    ParseResult::OK
  } else {
    stack.reverse();
    ParseResult::Incomplete{expect: stack}
  }
}

pub fn generator(data: &str) -> Vec<ParseResult> {
  data.lines()
    .map(|x| x.trim())
    .filter(|x| x.len() > 0)
    .map(|x| parse(&x))
    .collect()
}

pub fn part1(input: &Vec<ParseResult>) -> u64 {
  input.iter()
    .map(|r| match r {
      ParseResult::Corrupted{_expect: _, found: ch} => score(*ch),
      _ => 0 })
    .sum()
}

pub fn part2(input: &Vec<ParseResult>) -> u64 {
  let mut fix: Vec<u64> = input.iter()
    .map(|r| match r {
      ParseResult::Incomplete{expect: e} => fix_score(e),
      _ => 0 })
    .filter(|x| *x > 0)
    .collect();
  fix.sort();
  fix[fix.len() /2]
}
