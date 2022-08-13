use std::collections::HashMap;
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

  const SIZE: usize = 4;
  fn index(&self) -> usize {
    match self {
      Self::W => 0,
      Self::X => 1,
      Self::Y => 2,
      Self::Z => 3,
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
  register: [i64; Register::SIZE],
}

impl State {
  fn get_value(&self, op: &Operand) -> i64 {
    match op {
      Operand::Value(i) => *i,
      Operand::Register(reg) => self.register[reg.index()],
    }
  }

  // Evaluate the operation in the given state and input.
  // Updates the state.
  fn evaluate(&mut self, program: &[Operation], input: &[i64]) -> i64 {
    if program.len() == 0 {
      self.register[Register::SIZE - 1]
    } else {
      match &program[0] {
        Operation::Input(id, reg) =>
          self.register[reg.index()] = input[*id],
        Operation::Add(reg, operand) =>
          self.register[reg.index()] = self.register[reg.index()] + self.get_value(operand),
        Operation::Multiply(reg, operand) =>
          self.register[reg.index()] = self.register[reg.index()] * self.get_value(operand),
        Operation::Divide(reg, operand) =>
          self.register[reg.index()] = self.register[reg.index()] / self.get_value(operand),
        Operation::Modulo(reg, operand) =>
          self.register[reg.index()] = self.register[reg.index()] % self.get_value(operand),
        Operation::Equal(reg, operand) =>
          self.register[reg.index()] =
              if self.register[reg.index()] == self.get_value(operand) {1} else {0},
      }
      self.evaluate(&program[1..], input)
    }
  }
}

#[derive(Clone, Debug)]
struct InputDescriptor {
  inputs: Vec<u64>,
}

impl InputDescriptor {
  fn add_input_value(&mut self, position: usize, value: i64) {

  }
}
#[derive(Clone, Debug)]
struct SymbolicState {
  registers: [HashMap<i64, InputDescriptor>; Register::SIZE],
}

fn main() {
  let program = Operation::parse(&mut io::stdin().lock().lines());
  let input = vec![3,9,9,9,9,6,9,8,7,9,9,4,2,9];
  let mut state = State::default();
  state.evaluate(&program, &input);
  println!("result = {:?}", &state);
}
