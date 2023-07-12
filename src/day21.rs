use std::cmp::Reverse;

use priority_queue::PriorityQueue;

// a list of the roll and how often
fn die_rolls() -> Vec<(u64, u64)> {
  vec![(3,1), (4, 3), (5, 6), (6, 7), (7, 6), (8, 3), (9, 1)]
}

// Use the reversed scores as a priority so that we will
// advance the lower scores first to reuse states as much
// as possible.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Priority {
  high_score: u64,
  low_score: u64,
  time_lines: u64,
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Player {
  position: u64,
  score: u64,
}

impl Player {
  const BOARD_SIZE: u64 = 10;

  fn parse(line: &str) -> Self {
    let parts: Vec<&str> = line.split_ascii_whitespace().collect();
    let posn = parts[4].parse::<u64>().unwrap();
    Player{position: posn, score: 0}
  }

  fn advance(&mut self, spaces: u64) {
    self.position = ((self.position - 1 + spaces) % Player::BOARD_SIZE) + 1;
    self.score += self.position;
  }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Game {
  players: Vec<Player>,
  next: usize,
}

impl Game {
  fn parse(input: &mut dyn Iterator<Item = &str>) -> Self {
    let players: Vec<Player> = input.map(|l| Player::parse(l)).collect();
    Game{ players, next: 0 }
  }

  fn turn(&mut self, spaces: u64) {
    self.players[self.next].advance(spaces);
    self.next = (self.next + 1) % self.players.len();
  }

  const PART1_MAX_SCORE: u64 = 1000;
  const PART2_MAX_SCORE: u64 = 21;
  
  fn is_over(&self, max_score: u64) -> bool {
    self.players.iter()
      .map(|p| p.score).max().unwrap() >= max_score
  }

  fn make_priority(&self, times: u64) -> Reverse<Priority> {
    let mut max: u64 = 0;
    let mut min: u64 = u64::MAX;
    for p in &self.players {
      max = u64::max(max, p.score);
      min = u64::min(min, p.score);
    }
    Reverse(Priority{high_score: max, low_score: min, time_lines: times})
  }
}

pub fn generator(data: &str) -> Game {
  Game::parse(&mut data.lines()
    .map(|x| x.trim())
    .filter(|x| x.len() > 0))
}

pub fn part1(input: &Game) -> u64 {
  let mut next = 0;
  let mut game = input.clone();
  let mut die = Die::new();
  while !game.is_over(Game::PART1_MAX_SCORE) {
    game.players[next].advance(die.next() + die.next() + die.next());
    next = (next + 1) % game.players.len();
  }
  die.throws * game.players[next].score
}

pub fn part2(game: &Game) -> u64 {
  let mut queue: PriorityQueue<Game, Reverse<Priority>> = PriorityQueue::new();

  // push the initial state on to the queue
  let priority = game.make_priority(1);
  queue.push(game.clone(), priority);

  loop {
    // keep going until all games have been won
    if let Some((_, Reverse(priority))) = queue.peek() {
      if priority.high_score >= Game::PART2_MAX_SCORE {
        break;
      }
    }
    let (game, priority) = queue.pop().unwrap();

    // for each roll, update the board and put it back on the queue
    for (roll, times) in die_rolls() {
      let mut new_state = game.clone();
      new_state.turn(roll);
      let mut new_priority =
        new_state.make_priority(times * priority.0.time_lines);

      // if it is already on the queue, just merge them together
      if let Some(Reverse(prev)) = queue.get_priority(&new_state) {
        new_priority.0.time_lines += prev.time_lines;
        queue.change_priority(&new_state, new_priority);
      } else {
        queue.push(new_state, new_priority);
      }
    }
  }

  let mut wins: Vec<u64> = vec![0; 2];
  for (game, Reverse(priority)) in &queue {
    wins[game.next] += priority.time_lines;
  }
  *(wins.iter().max().unwrap())
}