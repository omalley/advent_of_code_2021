#[derive(Debug)]
pub struct Board {
  numbers: Vec<Vec<i32>>,
}

impl Board {
  const BOARD_SIZE: usize = 5;

  fn parse(input: &str) -> Self {
    let numbers: Vec<Vec<i32>> = input.lines()
      .map(|x| x.split_whitespace()
        .map(|x| x.parse::<i32>().unwrap())
        .collect())
      .collect();
    assert_eq!(numbers.len(), Self::BOARD_SIZE);
    assert_eq!(numbers.iter()
      .map(|x| x.len()).sum::<usize>(), Self::BOARD_SIZE * Self::BOARD_SIZE);
    Board{numbers}
  }
}

#[derive(Debug)]
pub struct Bingo {
  moves: Vec<i32>,
  boards: Vec<Board>,
}

impl Bingo {
  fn parse(input: &str) -> Self {
    let mut sections = input.split("\n\n");
    // Read the first section as a comma separated list of numbers.
    let moves = sections.next().unwrap()
      .split(",").map(|x| x.trim().parse::<i32>().unwrap())
      .collect();
    let boards = sections.map(|x| Board::parse(x)).collect();
    Bingo{moves, boards}
  }
}

#[derive(Debug)]
struct MarkedBoard<'a> {
  board: &'a Board,
  mark: [[bool; Board::BOARD_SIZE]; Board::BOARD_SIZE],
}

impl<'a> MarkedBoard<'a> {
  fn new(board: &'a Board) -> Self {
    let mark = [[false; Board::BOARD_SIZE]; Board::BOARD_SIZE];
    MarkedBoard{board, mark}
  }

  fn won(&self) -> bool {
    // look for winning rows
    for x in 0..Board::BOARD_SIZE {
      for y in 0..Board::BOARD_SIZE {
        if !self.mark[x][y] {
          break;
        }
        if y == Board::BOARD_SIZE - 1 {
          return true
        }
      }
    }

    // look for winning columns
    for y in 0..Board::BOARD_SIZE {
      for x in 0..Board::BOARD_SIZE {
        if !self.mark[x][y] {
          break;
        }
        if x == Board::BOARD_SIZE - 1 {
          return true
        }
      }
    }
    false
  }

  fn mark(&mut self, num: i32) {
    for x in 0..Board::BOARD_SIZE {
      for y in 0..Board::BOARD_SIZE {
        if self.board.numbers[x][y] == num {
          self.mark[x][y] = true;
        }
      }
    }
  }

  fn score(&self, num: i32) -> i64 {
    let mut sum : i64 = 0;
    for x in 0..Board::BOARD_SIZE {
      for y in 0..Board::BOARD_SIZE {
        if !self.mark[x][y] {
          sum += self.board.numbers[x][y] as i64;
        }
      }
    }
    sum * num as i64
  }
}

pub fn generator(data: &str) -> Bingo {
  Bingo::parse(data)
}

pub fn part1(bingo: &Bingo) -> i64 {
  let mut boards: Vec<MarkedBoard> =
    bingo.boards.iter().map(|b| MarkedBoard::new(b))
      .collect();
  for m in &bingo.moves {
    for b in &mut boards {
      b.mark(*m);
      if b.won() {
        return b.score(*m)
      }
    }
  }
  0
}

pub fn part2(bingo: &Bingo) -> i64 {
  let mut boards: Vec<MarkedBoard> =
    bingo.boards.iter().map(|b| MarkedBoard::new(b))
      .collect();
  for m in &bingo.moves {
    for b in &mut *boards {
      b.mark(*m);
    }
    if boards.len() == 1 && boards[0].won() {
      return boards[0].score(*m)
    }
    boards.retain(|x| !x.won());
  }
  0
}
