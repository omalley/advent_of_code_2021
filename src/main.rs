use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufRead, Error, ErrorKind};
use std::rc::Rc;

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

impl Display for Register {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Register::W => write!(f, "W"),
      Register::X => write!(f, "X"),
      Register::Y => write!(f, "Y"),
      Register::Z => write!(f, "Z"),
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

impl Display for Operand {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Operand::Value(v) => write!(f, "{}", v),
      Operand::Register(r) => write!(f, "{}", r),
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
  Equal(usize, Register, Operand),
}

impl Operation {
  fn parse_statement(line: &str,
                     next_input: &mut dyn Iterator<Item=usize>,
                     next_equal: &mut dyn Iterator<Item=usize>) -> io::Result<Self> {
    let words: Vec<String> = line.split_ascii_whitespace()
        .map(|x| String::from(x)).collect();
    let register = Register::parse(&words[1])?;
    match words[0].as_str() {
      "inp" => Ok(Self::Input(next_input.next().unwrap(), register)),
      "add" => Ok(Self::Add(register, Operand::parse(&words[2])?)),
      "mul" => Ok(Self::Multiply(register, Operand::parse(&words[2])?)),
      "div" => Ok(Self::Divide(register, Operand::parse(&words[2])?)),
      "mod" => Ok(Self::Modulo(register, Operand::parse(&words[2])?)),
      "eql" => Ok(Self::Equal(next_equal.next().unwrap(),
                              register, Operand::parse(&words[2])?)),
      _ => Err(Error::new(ErrorKind::Other, format!("Unknown operator {}", words[0]))),
    }
  }

  fn parse(lines: &mut dyn Iterator<Item=io::Result<String>>) -> io::Result<Vec<Self>> {
    let mut next_input = 0..usize::MAX;
    let mut next_equal = 0..usize::MAX;
    let mut result = Vec::new();
    for l in lines {
      match l {
        Ok(s) => {
          if s.len() > 0 {
            result.push(Self::parse_statement(&s, &mut next_input,
                                              &mut next_equal)?);
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

  fn get_register(&self) -> &Register {
    match self {
      Operation::Input(_, reg) => reg,
      Operation::Add(reg, _) => reg,
      Operation::Multiply(reg, _) => reg,
      Operation::Divide(reg, _) => reg,
      Operation::Modulo(reg, _) => reg,
      Operation::Equal(_, reg, _) => reg,
    }
  }
}

impl Display for Operation {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Operation::Input(idx, reg) => write!(f, "inp_{}({})", idx, reg),
      Operation::Add(left, right) => write!(f, "add({}, {})", left, right),
      Operation::Multiply(left, right) => write!(f, "mul({}, {})", left, right),
      Operation::Divide(left, right) => write!(f, "div({}, {})", left, right),
      Operation::Modulo(left, right) => write!(f, "mod({}, {})", left, right),
      Operation::Equal(idx, left, right) => write!(f, "eql_{}({}, {})", idx, left, right),
    }
  }
}

trait Environment {
  fn get_input(&self, id: usize) -> Vec<i64>;
  fn should_abandon(&self, op: &Operation, result: i64) -> bool;
}

// A simple environment that runs a single input
struct SimpleEnvironment {
  inputs: Vec<i64>,
}

impl Environment for SimpleEnvironment {
  fn get_input(&self, id: usize) -> Vec<i64> {
    if id < self.inputs.len() {
      return vec!{self.inputs[id]}
    }
    vec!{}
  }

  fn should_abandon(&self, _: &Operation, _: i64) -> bool {
    false
  }
}

type ExecutionResult = Result<(), String>;

#[derive(Clone, Debug, Default, PartialEq)]
struct State {
  register: [i64; Register::SIZE],
  inputs: Vec<i64>,
  pc: usize,
}

impl State {
  fn get_value(&self, op: &Operand) -> i64 {
    match op {
      Operand::Value(i) => *i,
      Operand::Register(reg) => self.register[reg.index()],
    }
  }

  fn do_input(&mut self, program: &[Operation], env: &dyn Environment) -> ExecutionResult {
    if let Operation::Input(id, reg) = &program[self.pc] {
      for input in env.get_input(*id) {
        let mut child_state = self.clone();
        child_state.inputs.push(input);
        child_state.register[reg.index()] = input;
        child_state.pc += 1;
        if child_state.execute(program, env).is_ok() {
          *self = child_state;
          return Ok(())
        }
      }
    }
    Err("Input exhausted".to_string())
  }

  /// Evaluate the program given an environment.
  /// Mutates the state.
  fn execute(&mut self, program: &[Operation], env: & dyn Environment) -> ExecutionResult {
    while self.pc < program.len() {
      let result: i64;
      let statement = &program[self.pc];
      match statement {
        Operation::Input(_, _) => {
          self.do_input(program, env)?;
          continue
        }
        Operation::Add(reg, operand) =>
          result = self.register[reg.index()] + self.get_value(operand),
        Operation::Multiply(reg, operand) =>
          result = self.register[reg.index()] * self.get_value(operand),
        Operation::Divide(reg, operand) =>
          result = self.register[reg.index()] / self.get_value(operand),
        Operation::Modulo(reg, operand) =>
          result = self.register[reg.index()] % self.get_value(operand),
        Operation::Equal(_, reg, operand) =>
          result = if self.register[reg.index()] == self.get_value(operand) {1} else {0},
      }
      self.register[statement.get_register().index()] = result;
      if env.should_abandon(statement, result) {
        return Err(statement.to_string() + " abandoned")
      }
      self.pc += 1;
    }
    Ok(())
  }
}

/// A symbolic representation of a boolean value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SymbolicBoolean {
  ANY,
  TRUE,
  FALSE,
  INVALID,
}

impl SymbolicBoolean {
  fn or(self, other: Self) -> Self {
    if self == other {
      self
    } else if self == Self::INVALID {
      other
    } else if other == Self::INVALID {
      self
    } else {
      Self::ANY
    }
  }

  fn and(self, other: Self) -> Self {
    if self == other {
      self
    } else if self == Self::ANY {
      other
    } else if other == Self::ANY {
      self
    } else {
      Self::INVALID
    }
  }

  /// Get the value, if it is unique.
  fn get_single(self) -> Option<bool> {
    match self {
      Self::TRUE => Some(true),
      Self::FALSE => Some(false),
      _ => None,
    }
  }
}

/// Tracking information for each value.
/// We keep track of whether each equals op-code evaluated to true or false
#[derive(Clone, Debug, Default)]
struct BreadCrumb {
  /// The output of each equals operator
  /// Values past the end of the vector are SymbolicValue::ANY.
  crumbs: Vec<SymbolicBoolean>,
}

impl BreadCrumb {
  fn init(num_variables: usize) -> Self {
    BreadCrumb{crumbs: Vec::with_capacity(num_variables)}
  }

  /// Replaces the previous value for that location with the given
  /// value.
  fn set(&mut self, equal_idx: usize, value: bool) {
    while self.crumbs.len() <= equal_idx {
      self.crumbs.push(SymbolicBoolean::ANY);
    }
    self.crumbs[equal_idx] =
      if value { SymbolicBoolean::TRUE } else { SymbolicBoolean::FALSE };
  }

  /// Ors the other into self.
  fn or(&mut self, other: &Self) -> &mut Self{
    let both = usize::min(self.crumbs.len(), other.crumbs.len());
    self.crumbs.reserve(other.crumbs.len() - both);
    for i in 0..both {
      self.crumbs[i] = self.crumbs[i].or(other.crumbs[i]);
    }
    // handle the extra values in self
    for i in both..self.crumbs.len() {
      self.crumbs[i] = SymbolicBoolean::ANY.or(self.crumbs[i]);
    }
    // handle the extra values in other
    for i in both..other.crumbs.len() {
      self.crumbs.push(SymbolicBoolean::ANY.or(other.crumbs[i]));
    }
    self
  }

  /// Ands the other into self.
  fn and(&mut self, other: &Self) -> &mut Self {
    let both = usize::min(self.crumbs.len(), other.crumbs.len());
    self.crumbs.reserve(other.crumbs.len() - both);
    for i in 0..both {
      self.crumbs[i] = self.crumbs[i].and(other.crumbs[i]);
    }
    // The extra values in self don't change and the extra ones in other are copied.
    // ANY.and(x) -> x
    for i in both..other.crumbs.len() {
      self.crumbs.push(other.crumbs[i]);
    }
    self
  }

  /// Is this a valid value?
  fn is_valid(&self) -> bool {
    self.crumbs.iter().find(|&v| *v == SymbolicBoolean::INVALID).is_none()
  }

  /// Build the list of constraints that lead to the right answer.
  /// If a position is Some, that equals operator must have that result.
  fn get_constraint(&self) -> Vec<Option<bool>> {
    self.crumbs.iter()
      .map(|&v| v.get_single())
      .collect()
  }
}

impl Display for BreadCrumb {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let parts: Vec<String> = self.crumbs.iter().enumerate()
      .filter(|(_, &v)| v != SymbolicBoolean::ANY)
      .map(|(i,v)| format!("eql_{}: {:?}", i, v))
      .collect();
    write!(f, "{}", parts.join("; "))
  }
}

#[derive(Clone, Debug, Default)]
struct SymbolicValue {
  // for each value, track the constraints that got us there
  values: HashMap<i64, BreadCrumb>,
}

impl SymbolicValue {
  fn literal(x: i64) -> Self {
    let mut values = HashMap::default();
    values.insert(x, BreadCrumb::init(0));
    Self{values}
  }

  fn values(&self) -> Vec<i64> {
    let mut result = self.values.keys().cloned().collect::<Vec<i64>>();
    result.sort();
    result
  }
}

impl Display for SymbolicValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let result = self.values().iter()
        .map(| val | format!("{}: [{}]", val, self.values.get(val).unwrap()))
        .collect::<Vec<String>>();
    write!(f, "{}", result.join("; "))
  }
}

#[derive(Clone, Debug)]
struct SymbolicState {
  pc: usize,
  register: [Rc<SymbolicValue>; Register::SIZE],
}

impl SymbolicState {
  fn default() -> Self {
    let zero = Rc::new(SymbolicValue::literal(0));
    let register: [Rc<SymbolicValue>; Register::SIZE] = [(); Register::SIZE].map(|_| zero.clone());
    SymbolicState{pc: 0, register}
  }

  fn get_value(&self, opd: &Operand) -> Rc<SymbolicValue> {
    match opd {
      Operand::Register(reg) => self.register[reg.index()].clone(),
      Operand::Value(x) => Rc::new(
        SymbolicValue::literal(*x)),
    }
  }

  /// Generate all possible values for an input statement
  fn do_input(&self) -> Rc<SymbolicValue> {
    // generate all values from 1 to 9
    Rc::new(SymbolicValue{values: (1 ..= 9)
        .map(|v| (v, BreadCrumb::init(0)))
        .collect()})
  }

  fn do_add(&self, reg: &Register, opd: &Operand) -> Rc<SymbolicValue> {
    let left = &self.register[reg.index()];
    let right = self.get_value(opd);
    let mut result = SymbolicValue::default();
    for (left_val, left_descr) in &left.values {
      for (right_val, right_descr) in &right.values {
        let total_val = left_val + right_val;
        let mut total_descr = left_descr.clone();
        total_descr.and(right_descr);
        match result.values.get_mut(&total_val) {
          Some(old_descr) => { old_descr.or(&total_descr); },
          None => { result.values.insert(total_val, total_descr); },
        }
      }
    }
    Rc::new(result)
  }

  fn do_multiply(&self, reg: &Register, opd: &Operand) -> Rc<SymbolicValue> {
    let left = &self.register[reg.index()];
    let right = self.get_value(opd);
    let mut result = SymbolicValue::default();
    for (&left_val, left_descr) in &left.values {
      for (&right_val, right_descr) in &right.values {
        let total_val = left_val * right_val;
        let mut total_descr;
        // depending on the zeros, figure out the requirements
        if left_val == 0 && right_val == 0 {
          total_descr = left_descr.clone();
          total_descr.or(right_descr);
        } else if left_val == 0 {
          total_descr = left_descr.clone();
        } else if right_val == 0 {
          total_descr = right_descr.clone();
        } else {
          total_descr = left_descr.clone();
          total_descr.and(right_descr);
        }
        match result.values.get_mut(&total_val) {
          Some(old_descr) => { old_descr.or(&total_descr); },
          None => { result.values.insert(total_val, total_descr); },
        }
      }
    }
    Rc::new(result)
  }

  fn do_divide(&self, reg: &Register, opd: &Operand) -> Rc<SymbolicValue> {
    let left = &self.register[reg.index()];
    let right = self.get_value(opd);
    let mut result = SymbolicValue::default();
    for (&left_val, left_descr) in &left.values {
      for (&right_val, right_descr) in &right.values {
        let total_val = left_val / right_val;
        let mut total_descr = left_descr.clone();
        // If the left value is 0, we don't care about the right
        if left_val != 0 {
          total_descr.and(right_descr);
        }
        match result.values.get_mut(&total_val) {
          Some(old_descr) => { old_descr.or(&total_descr); },
          None => { result.values.insert(total_val, total_descr); },
        }
      }
    }
    Rc::new(result)
  }

  fn do_modulo(&self, reg: &Register, opd: &Operand) -> Rc<SymbolicValue> {
    let left = &self.register[reg.index()];
    let right = self.get_value(opd);
    let mut result = SymbolicValue::default();
    for (&left_val, left_descr) in &left.values {
      for (&right_val, right_descr) in &right.values {
        let total_val = left_val % right_val;
        let mut total_descr = left_descr.clone();
        // If the left value is 0, we don't care about the right
        if left_val != 0 {
          total_descr.and(right_descr);
        }
        match result.values.get_mut(&total_val) {
          Some(old_descr) => { old_descr.or(&total_descr); },
          None => { result.values.insert(total_val, total_descr); },
        }
      }
    }
    Rc::new(result)
  }

  fn do_equals(&self, id: usize, reg: &Register, opd: &Operand) -> Rc<SymbolicValue> {
    let left = &self.register[reg.index()];
    let right = self.get_value(opd);
    let mut result = SymbolicValue::default();
    for (left_val, left_descr) in &left.values {
      for (right_val, right_descr) in &right.values {
        let total_val = if left_val == right_val { 1 } else { 0 };
        let mut total_descr = left_descr.clone();
        total_descr.and(right_descr);
        match result.values.get_mut(&total_val) {
          Some(old_descr) => { old_descr.or(&total_descr); },
          None => { result.values.insert(total_val, total_descr); },
        }
      }
    }
    // Set the bread crumb for which branch was taken
    for (&val, descr) in result.values.iter_mut() {
      descr.set(id, val == 1);
    }
    Rc::new(result)
  }

  // Evaluate the operation in the given state and input.
  // Updates the state.
  fn evaluate(&mut self, program: &[Operation]) {
    if program.len() > 0 {
      match &program[0] {
        Operation::Input(_, reg) =>
          self.register[reg.index()] = self.do_input(),
        Operation::Add(reg, operand) =>
          self.register[reg.index()] = self.do_add(reg, operand),
        Operation::Multiply(reg, operand) =>
          self.register[reg.index()] = self.do_multiply(reg, operand),
        Operation::Divide(reg, operand) =>
          self.register[reg.index()] = self.do_divide(reg, operand),
        Operation::Modulo(reg, operand) =>
          self.register[reg.index()] = self.do_modulo(reg, operand),
        Operation::Equal(id, reg, operand) =>
          self.register[reg.index()] = self.do_equals(*id, reg, operand),
      }
      self.pc += 1;
      self.evaluate(&program[1..])
    }
  }
}

/// An environment that tries each input value, but has
/// constraints on the results of equals.
struct ConstrainedEnvironment {
  constraint: Vec<Option<bool>>,
}

impl Environment for ConstrainedEnvironment {
  fn get_input(&self, _: usize) -> Vec<i64> {
    (1..=9).rev().collect()
  }

  fn should_abandon(&self, op: &Operation, result: i64) -> bool {
    if let Operation::Equal(id, _, _) = op {
      if *id < self.constraint.len() {
        if let Some(required) = self.constraint[*id] {
          if required != (result == 1) {
            return true
          }
        }
      }
    }
    false
  }
}

fn main() {
  let program = Operation::parse_file("input24.txt").unwrap();
  let mut state = SymbolicState::default();
  state.evaluate(&program);
  match state.register[Register::Z.index()].values.get(&0) {
    Some(sol) => println!("Solution = {}", sol),
    None => println!("Nothing!"),
  }
}

#[cfg(test)]
mod tests {
  use crate::{BreadCrumb, ConstrainedEnvironment, Operand, Operation, Register, SimpleEnvironment, State, SymbolicState};

  #[test]
  fn test_little_execution() {
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
    let inputs = vec![9];
    let mut state = State::default();
    let env = SimpleEnvironment{inputs};
    assert!(state.execute(&program, &env).is_ok());
    assert_eq!([1, 0, 0, 1], state.register);
  }

  #[test]
  fn test_execution() {
    let program = Operation::parse_file("input24.txt").unwrap();
    let inputs = vec![3,9,9,9,9,6,9,8,7,9,9,4,2,9];
    let env = SimpleEnvironment{inputs};
    let mut state = State::default();
    assert!(state.execute(&program, &env).is_ok());
    assert_eq!([9, 0, 0, 0], state.register);
  }

  #[test]
  fn test_constrained_execution() {
    let text = vec!{
      "inp w",
      "mul w 10",
      "inp x",
      "add w x",
      "add y w",
      "eql y 56"};
    let program = Operation::parse(&mut text.iter()
      .map(|l| Ok(l.to_string()))).unwrap();
    let env = ConstrainedEnvironment{constraint: vec!{Some(true)}};
    let mut state = State::default();
    assert!(state.execute(&program, &env).is_ok());
    assert_eq!([56, 6, 1, 0], state.register);
  }

  #[test]
  fn test_breadcrumbs() {
    let mut descr = BreadCrumb::init(14);
    descr.set(1, false);
    assert_eq!("eql_1: FALSE", descr.to_string());
    descr.set(2, false);
    descr.set(4, true);
    assert_eq!("eql_1: FALSE; eql_2: FALSE; eql_4: TRUE", descr.to_string());
    let mut descr2 = BreadCrumb::init(14);
    descr2.set(2, true);
    assert_eq!("eql_2: TRUE", descr2.to_string());
    descr.and(&descr2);
    assert_eq!("eql_1: FALSE; eql_2: INVALID; eql_4: TRUE", descr.to_string());
  }

  #[test]
  fn test_symbolic() {
    let mut state = SymbolicState::default();
    state.register[0] = state.do_input();
    state.register[1] = state.do_input();
    state.register[2] = state.do_add(&Register::W, &Operand::Register(Register::X));
    for sv in &state.register {
      println!("sv = {}", sv);
    }
    assert_eq!((2..=18).collect::<Vec<i64>>(), state.register[2].values());
  }

  #[test]
  fn test_symbolic_little() {
    let program = Operation::parse_file("little.txt").unwrap();
    let mut state = SymbolicState::default();
    state.evaluate(&program);
    assert_eq!(vec![0, 1], state.register[0].values());
    assert_eq!(vec![0, 1], state.register[3].values());
  }
}