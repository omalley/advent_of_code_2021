use std::io;
use std::io::{BufRead, Error};

#[derive (Debug)]
enum Register {
  W,
  X,
  Y,
  Z,
}

impl Register {
  fn parse(line: &str) -> Result<Self, String> {
    match line {
      "w" => Ok(Self::W),
      "x" => Ok(Self::X),
      "y" => Ok(Self::Y),
      "z" => Ok(Self::Z),
      _ => Err(format!("Unknown register {}", line)),
    }
  }
}

#[derive(Debug)]
enum Operand {
  Value(i64),
  Register(Register),
}

impl Operand {
  fn parse(line: &str) -> Result<Self, String> {
    if let Ok(val) = line.parse::<i64>() {
      Ok(Self::Value(val))
    } else {
      Ok(Self::Register(Register::parse(line)?))
    }
  }
}

#[derive(Debug)]
enum Operation {
  Input(usize, Register),
  Add(Register, Operand),
  Multiply(Register, Operand),
  Divide(Register, Operand),
  Modulo(Register, Operand),
  Equal(Register, Operand),
}

impl Operation {
  fn parse_statement(line: &str, next_input: &mut usize) -> Result<Self, String> {
    let words: Vec<String> = line.split_ascii_whitespace()
        .map(|x| String::from(x)).collect();
    let register = Register::parse(&words[1])?;
    match words[0].as_str() {
      "inp" => {
        let id = *next_input;
        *next_input += 1;
        Ok(Self::Input(id, register))
      },
      "add" => Ok(Self::Add(register, Operand::parse(&words[2])?)),
      "mul" => Ok(Self::Multiply(register, Operand::parse(&words[2])?)),
      "div" => Ok(Self::Divide(register, Operand::parse(&words[2])?)),
      "mod" => Ok(Self::Modulo(register, Operand::parse(&words[2])?)),
      "eql" => Ok(Self::Equal(register, Operand::parse(&words[2])?)),
      _ => Err(format!("Unknown operator {}", words[0]))
    }
  }

  fn parse(lines: &mut dyn Iterator<Item=Result<String,Error>>) -> Vec<Self> {
    let mut next_input = 0;
    lines.map(|x| String::from(x.unwrap()))
        .filter(|x| x.len() > 0)
        .map(|x| Operation::parse_statement(&x, &mut next_input).unwrap())
        .collect()
  }
}

#[derive(Clone, Debug, Default)]
struct State {
  w: i64,
  x: i64,
  y: i64,
  z: i64,
}

impl State {
  fn get_ref(&mut self, reg: &Register) -> &mut i64 {
    match reg {
      Register::W => &mut self.w,
      Register::X => &mut self.x,
      Register::Y => &mut self.y,
      Register::Z => &mut self.z,
    }
  }

  fn get_value(&self, op: &Operand) -> i64 {
    match op {
      Operand::Value(i) => *i,
      Operand::Register(reg) =>
        match reg {
          Register::W => self.w,
          Register::X => self.x,
          Register::Y => self.y,
          Register::Z => self.z,
        }
    }
  }

  // Evaluate the operation in the given state and input.
  // Updates the state.
  fn evaluate(&mut self, program: &[Operation], input: &[i64]) -> i64 {
    if program.len() == 0 {
      self.z
    } else {
      match &program[0] {
        Operation::Input(id, reg) =>
          *self.get_ref(reg) = input[*id],
        Operation::Add(reg, operand) =>
          *self.get_ref(reg) = *self.get_ref(reg) + self.get_value(operand),
        Operation::Multiply(reg, operand) =>
          *self.get_ref(reg) = *self.get_ref(reg) * self.get_value(operand),
        Operation::Divide(reg, operand) =>
          *self.get_ref(reg) = *self.get_ref(reg) / self.get_value(operand),
        Operation::Modulo(reg, operand) =>
          *self.get_ref(reg) = *self.get_ref(reg) % self.get_value(operand),
        Operation::Equal(reg, operand) =>
          *self.get_ref(reg) =
              if *self.get_ref(reg) == self.get_value(operand) {1} else {0},
      }
      self.evaluate(&program[1..], input)
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PotentialValue {
  value: i64,
  inputs: Vec<u64>,
}

fn main() {
  let program = Operation::parse(&mut io::stdin().lock().lines());
  let input = vec![3,9,9,9,9,6,9,8,7,9,9,4,2,9];
  let mut state = State::default();
  state.evaluate(&program, &input);
  println!("result = {:?}", &state);
}
