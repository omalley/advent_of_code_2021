use std::io;
use std::io::BufRead;
use std::iter;

fn main() {
  let stdin = io::stdin();
  let mut stream = stdin.lock().lines()
     .map(|x| String::from(x.unwrap().trim()))
     .filter(|x| x.len() > 0);

  // read the first line as a sequence of integers
  let moves: Vec<i32> = stream.next().unwrap().split(",")
     .map(|x| x.trim().parse::<i32>().unwrap()).collect();

  // read the rest as boards
  let mut boards = read_board_list(stream.by_ref());
  
  println!("score = {}", run_moves(&moves, &mut boards));
}

const BOARD_SIZE: usize = 5;

#[derive(Debug, Default)]
struct BingoBoard {
  board: Vec<Vec<i32>>,
  mark: [[bool; BOARD_SIZE]; BOARD_SIZE],
}

impl BingoBoard {
  fn won(&self) -> bool {
    // look for winning rows
    for x in 0..BOARD_SIZE {
      for y in 0..BOARD_SIZE {
        if !self.mark[x][y] {
          break;
        }
        if y == BOARD_SIZE - 1 {
          return true
        }
      }
    }

    // look for winning columns
    for y in 0..BOARD_SIZE {
      for x in 0..BOARD_SIZE {
        if !self.mark[x][y] {
          break;
        }
        if x == BOARD_SIZE - 1 {
          return true
        }
      }
    }
    false
  }

  fn mark(&mut self, num: i32) {
    for x in 0..BOARD_SIZE {
      for y in 0..BOARD_SIZE {
        if self.board[x][y] == num {
          self.mark[x][y] = true;
        }
      }
    }
  }

  fn score(&self, num: i32) -> i64 {
    let mut sum : i64 = 0;
    for x in 0..BOARD_SIZE {
      for y in 0..BOARD_SIZE {
        if !self.mark[x][y] {
          sum += self.board[x][y] as i64;
        }
      }
    }
    sum * num as i64
  }
}

fn read_board_list(stream: &mut dyn iter::Iterator<Item = String>)
    -> Vec<BingoBoard> {
  let mut peek = stream.peekable();
  let mut result = Vec::new();
  while peek.peek() != None {
    result.push(read_board(peek.by_ref()));
  }
  result
}

fn read_board(stream: &mut dyn iter::Iterator<Item = String>) -> BingoBoard {
  let mut result = BingoBoard::default();
  for _ in 0..BOARD_SIZE {
    let row: Vec<i32> = stream.next().unwrap().split_whitespace()
       .map(|x| x.parse::<i32>().unwrap()).collect();
    assert!(row.len() == BOARD_SIZE);
    result.board.push(row);
  }
  assert!(result.board.len() == BOARD_SIZE);
  result
}

fn run_moves(moves: &Vec<i32>, boards: &mut Vec<BingoBoard>) -> i64 {
  for m in moves {
    println!("Making move {}", *m);
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
