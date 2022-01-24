use std::io;
use std::io::BufRead;

fn main() {
  let stdin = io::stdin();
  let mut game = Game::parse(&mut stdin.lock().lines()
       .map(|x| String::from(x.unwrap().trim()))
       .filter(|x| x.len() > 0));
  println!("game = {:?}", game);
  let mut next = 0;
  while !game.is_over() {
    game.players[next].turn(&mut game.die);
    next = (next + 1) % game.players.len();
  }
  println!("game = {:?}", game);
  println!("result = {}", game.die.throws * game.players[next].score);
}

#[derive(Debug)]
struct Die {
  next: u64,
  throws: u64,
}

impl Die {
  fn new() -> Self {
    Die{ next: 1, throws: 0 }
  }

  const SIZE: u64 = 100;
  
  fn next(&mut self) -> u64 {
    let result = self.next;
    self.next = (self.next % Die::SIZE) + 1;
    self.throws += 1;
    result
  }
}

#[derive(Debug)]
struct Player {
  id: u64,
  position: u64,
  score: u64,
}

impl Player {
  const BOARD_SIZE: u64 = 10;

  fn parse(line: &str) -> Self {
    let parts: Vec<&str> = line.split_ascii_whitespace().collect();
    let id = parts[1].parse::<u64>().unwrap();
    let posn = parts[4].parse::<u64>().unwrap();
    return Player{id: id, position: posn, score: 0}
  }

  fn advance(&mut self, spaces: u64) {
    self.position = ((self.position - 1 + spaces) % Player::BOARD_SIZE) + 1;
    self.score += self.position;
  }

  fn turn(&mut self, die: &mut Die) {
    self.advance(die.next() + die.next() + die.next());
  }
}

#[derive(Debug)]
struct Game {
  players: Vec<Player>,
  die: Die,
}

impl Game {
  fn parse(input: &mut dyn Iterator<Item = String>) -> Self {
    let mut players: Vec<Player> = Vec::new();
    for line in input {
      players.push(Player::parse(&line));
    }
    Game{ players: players, die: Die::new() }
  }

  const MAX_SCORE: u64 = 1000;
  
  fn is_over(&self) -> bool {
    self.players.iter()
      .map(|p| p.score).reduce(|a, b| u64::max(a, b)).unwrap()
      >= Game::MAX_SCORE
  }
}

