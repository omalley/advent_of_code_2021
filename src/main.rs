use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, Error, ErrorKind};

#[derive (Debug)]
enum Register {
  W,
  X,
  Y,
  Z,
}

impl Register {
  fn parse(line: &str) -> io::Result<Self> {
    match line {
      "w" => Ok(Self::W),
      "x" => Ok(Self::X),
      "y" => Ok(Self::Y),
      "z" => Ok(Self::Z),
      _ => Err(Error::new(ErrorKind::Other, format!("Unknown register {}", line))),
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
  fn parse(line: &str) -> io::Result<Self> {
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
  fn parse_statement(line: &str, next_input: &mut usize) -> io::Result<Self> {
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
      _ => Err(Error::new(ErrorKind::Other, format!("Unknown operator {}", words[0]))),
    }
  }

  fn parse(lines: &mut dyn Iterator<Item=io::Result<String>>) -> io::Result<Vec<Self>> {
    let mut next_input = 0;
    let mut result = Vec::new();
    for l in lines {
      match l {
        Ok(s) => {
          if s.len() > 0 {
            result.push(Self::parse_statement(&s, &mut next_input)?)
          }
        },
        Err(e) => return Err(e),
      }
    }
    Ok(result)
  }

  fn parse_file(filename: &str) -> io::Result<Vec<Self>> {
    let file = File::open(filename)?;
    Self::parse(&mut io::BufReader::new(file).lines())
  }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct State {
  register: [i64; Register::SIZE],
}

impl State {
  fn init(w: i64, x: i64, y: i64, z: i64) -> Self {
    let register = [w, x, y, z];
    State{register}
  }

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

#[derive(Clone, Debug, Default)]
struct InputDescriptor {
  inputs: Vec<u64>,
}

impl InputDescriptor {
  fn init(input_idx: usize, value: i64) -> Self {
    let inputs = vec!{(value << (input_idx * 4)) as u64};
    InputDescriptor{inputs}
  }

  fn mark_input(&mut self, input_idx: usize, value: i64) {
    for val in self.inputs.iter_mut() {
      let mask = 0xf << (input_idx * 4);
      *val = (*val & !mask) | (value << (input_idx * 4)) as u64;
    }
  }
}

#[derive(Clone, Debug)]
struct SymbolicState {
  registers: [HashMap<i64, InputDescriptor>; Register::SIZE],
}

fn main() {
  let program = Operation::parse(&mut io::stdin().lock().lines()).unwrap();
  let input = vec![3,9,9,9,9,6,9,8,7,9,9,4,2,9];
  let mut state = State::default();
  state.evaluate(&program, &input);
  println!("result = {:?}", &state);
}

#[cfg(test)]
mod tests {
  use crate::{InputDescriptor, Operation, State};

  #[test]
  fn test_mini_execution() {
    let mut text = vec!{
      "inp w",
      "add z w",
      "mod z 2",
      "div w 2",
      "add y w",
      "mod y 2",
      "div w 2",
      "add x w",
      "mod x 2",
      "div w 2",
      "mod w 2"};
    let program = Operation::parse(&mut text.iter()
      .map(|l| Ok(l.to_string()))).unwrap();
    let input = vec![9];
    let mut state = State::default();
    state.evaluate(&program, &input);
    assert_eq!(State::init(1,0,0,1), state);
  }

  #[test]
  fn test_execution() {
    let program = Operation::parse_file("input24.txt").unwrap();
    let input = vec![3,9,9,9,9,6,9,8,7,9,9,4,2,9];
    let mut state = State::default();
    state.evaluate(&program, &input);
    assert_eq!(State::init(9, 0, 0, 0), state);
  }

  #[test]
  fn test_input_descriptor() {
    let mut descr = InputDescriptor::init(1, 2);
    assert_eq!(1, descr.inputs.len());
    assert_eq!(0x20, descr.inputs[0]);
    descr.mark_input(4, 3);
    assert_eq!(0x30020, descr.inputs[0]);
    descr.mark_input(4, 5);
    assert_eq!(0x50020, descr.inputs[0]);
  }
}