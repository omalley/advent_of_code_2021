use std::collections::HashMap;
use std::fmt::{Display, Formatter};
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
  // the index and the set register
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct InputAlternative {
  input: u64,
}

impl InputAlternative {
  fn init(input_idx: usize, value: i64) -> Self {
    assert!(value > 0 && value < 10);
    InputAlternative{input: (value << (input_idx * 4)) as u64}
  }

  fn and(&self, other: &Self) -> Option<Self> {
    // Check for conflicts
    let mut left = self.input;
    let mut right = other.input;
    while left != 0 && right != 0 {
      let left_digit = left & 0xf;
      let right_digit = right & 0xf;
      if left_digit != 0 && right_digit != 0 && left_digit != right_digit {
        return None
      }
      left = left >> 4;
      right = right >> 4;
    }
    Some(InputAlternative{input: self.input | other.input})
  }
}

impl Display for InputAlternative {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let mut result: Vec<String> = Vec::new();
    let mut num = self.input;
    let mut input_idx = 0;
    while num != 0 {
      let digit = num & 0xf;
      if digit != 0 {
        result.push(format!("{}: {}", input_idx, digit));
      }
      input_idx += 1;
      num = num >> 4;
    }
    write!(f, "{{{}}}", result.join(", "))
  }
}

#[derive(Clone, Debug)]
struct InputDescriptor {
  alternatives: Vec<InputAlternative>,
}

impl InputDescriptor {
  fn default() -> Self {
    InputDescriptor{alternatives: vec!{InputAlternative::default()}}
  }

  fn init(input_idx: usize, value: i64) -> Self {
    InputDescriptor{alternatives: vec!{InputAlternative::init(input_idx, value)}}
  }
}

impl Display for InputDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let result = self.alternatives.iter()
      .map(|s| s.to_string())
      .collect::<Vec<String>>();
    write!(f, "{{ {} }}", result.join(", "))
  }
}

#[derive(Clone, Debug, Default)]
struct SymbolicValue {
  values: HashMap<i64, InputDescriptor>,
}

impl SymbolicValue {
  fn literal(x: i64) -> Self {
    SymbolicValue{values: [(x, InputDescriptor::default())].iter().cloned().collect()}
  }

  fn values(&self) -> Vec<i64> {
    let mut result = self.values.keys().cloned().collect::<Vec<i64>>();
    result.sort();
    result
  }
}

#[derive(Clone, Debug)]
struct SymbolicState {
  register: [SymbolicValue; Register::SIZE],
}

impl SymbolicState {
  fn get_value(&self, opd: &Operand) -> SymbolicValue {
    match opd {
      Operand::Register(reg) => self.register[reg.index()].clone(),
      Operand::Value(x) => SymbolicValue::literal(*x),
    }
  }

  /// Generate all possible values for an input statement
  fn do_input(idx: usize) -> SymbolicValue {
    // generate all values from 1 to 9
    SymbolicValue{values: (1 .. 10)
      .map(|v| (v, InputDescriptor::init(idx, v)))
      .collect()}
  }

  fn do_add(&self, reg: &Register, opd: &Operand) -> SymbolicValue {
    let left = &self.register[reg.index()];
    let right = self.get_value(opd);
    let mut result = SymbolicValue::default();
    
    result
  }

  fn do_multiply(&self, reg: &Register, opd: &Operand) -> SymbolicValue {
    SymbolicValue::default()
  }

  fn do_divide(&self, reg: &Register, opd: &Operand) -> SymbolicValue {
    SymbolicValue::default()
  }

  fn do_modulo(&self, reg: &Register, opd: &Operand) -> SymbolicValue {
    SymbolicValue::default()
  }

  fn do_equals(&self, reg: &Register, opd: &Operand) -> SymbolicValue {
    SymbolicValue::default()
  }

  // Evaluate the operation in the given state and input.
  // Updates the state.
  fn evaluate(&mut self, program: &[Operation], input: &[i64]) {
    if program.len() > 0 {
      match &program[0] {
        Operation::Input(id, reg) =>
          self.register[reg.index()] = Self::do_input(*id),
        Operation::Add(reg, operand) =>
          self.register[reg.index()] = self.do_add(reg, operand),
        Operation::Multiply(reg, operand) =>
          self.register[reg.index()] = self.do_multiply(reg, operand),
        Operation::Divide(reg, operand) =>
          self.register[reg.index()] = self.do_divide(reg, operand),
        Operation::Modulo(reg, operand) =>
          self.register[reg.index()] = self.do_modulo(reg, operand),
        Operation::Equal(reg, operand) =>
          self.register[reg.index()] = self.do_equals(reg, operand),
      }
      self.evaluate(&program[1..], input)
    }
  }
}

fn main() {
  let program = Operation::parse_file("little.txt").unwrap();
  let stdin = io::stdin();
  let input: Vec<i64> = stdin.lock().lines()
          .map(|x| String::from(x.unwrap()))
          .filter(|x| x.len() > 0)
          .map(|x| x.parse::<i64>().unwrap())
          .collect();
  let mut state = State::default();
  state.evaluate(&program, &input);
  println!("result = {:?}", &state);
}

#[cfg(test)]
mod tests {
  use crate::{InputAlternative, Operation, State, SymbolicState, SymbolicValue};

  #[test]
  fn test_mini_execution() {
    let text = vec!{
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
  fn test_input_alternative() {
    let descr = InputAlternative::init(1, 2);
    assert_eq!("{1: 2}", descr.to_string());
    let result = InputAlternative::init(4, 3)
      .and(&descr).unwrap();
    assert_eq!("{1: 2, 4: 3}", result.to_string());
    let result = InputAlternative::init(4, 5).and(&result);
    assert_eq!(None, result);
  }

  #[test]
  fn test_symbolic() {
    let result = SymbolicValue::literal(12);
    let mut keys = result.keys().cloned().collect::<Vec<i64>>();
    keys.sort();
    assert_eq!(vec!{12}, keys);
    let result = SymbolicState::do_input(3);
    keys = result.keys().cloned().collect::<Vec<i64>>();
    keys.sort();
    assert_eq!((1..10).collect::<Vec<i64>>(), keys);
  }
}