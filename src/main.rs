use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let lines: Vec<Display> = stdin.lock().lines()
     .map(|x| String::from(x.unwrap().trim()))
     .filter(|x| x.len() > 0)
     .map(|x| Display::parse(&x))
     .collect();

  println!("linesn = {:?}", lines);
  let easy = lines.iter().map(|x| x.easy_numbers())
    .fold(0, |acc, x| acc + x);
  println!("easy = {}", easy);
}

#[derive(Debug,Default)]
struct Display {
  digits: Vec<String>,
  display: Vec<String>,
}

impl Display {
  fn parse(input: &str) -> Self {
    let mut result = Display::default();
    let parts: Vec<String> = input.split("|")
        .map(|x| String::from(x)).collect();
    result.digits = parts[0].split_whitespace().
        map(|x| sort_word(x.trim())).collect();
    result.display = parts[1].split_whitespace().
        map(|x| sort_word(x.trim())).collect();
    result
  }

  fn easy_numbers(&self) -> i32 {
    let easy = vec![2, 3, 4, 7];
    self.display.iter().filter(|x| easy.contains(&x.len())).count() as i32
  }
}

fn sort_word(input: &str) -> String {
  let mut chars: Vec<char> = input.chars().collect();
  chars.sort();
  String::from_iter(chars)
}