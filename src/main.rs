use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fmt;
use std::io;
use std::io::BufRead;
use std::rc::Rc;

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
  Input(Register),
  Add(Register, Operand),
  Multiply(Register, Operand),
  Divide(Register, Operand),
  Modulo(Register, Operand),
  Equal(Register, Operand),
}

impl Operation {
  fn parse(line: &str) -> Result<Self, String> {
    let words: Vec<String> = line.split_ascii_whitespace()
        .map(|x| String::from(x)).collect();
    let register = Register::parse(&words[1])?;
    match words[0].as_str() {
      "inp" => Ok(Self::Input(register)),
      "add" => Ok(Self::Add(register, Operand::parse(&words[2])?)),
      "mul" => Ok(Self::Multiply(register, Operand::parse(&words[2])?)),
      "div" => Ok(Self::Divide(register, Operand::parse(&words[2])?)),
      "mod" => Ok(Self::Modulo(register, Operand::parse(&words[2])?)),
      "eql" => Ok(Self::Equal(register, Operand::parse(&words[2])?)),
      _ => Err(format!("Unknown operator {}", words[0]))
    }
  }

  // Evaluate the operation in the given state and input.
  // Updates the state.
  fn evaluate(&self, state: &mut State, input: i64) -> i64 {
    match self {
      Self::Input(reg) => state.set(reg, input),
      Self::Add(reg, operand) =>
        state.set(reg, state.get(reg) + state.get_value(operand)),
      Self::Multiply(reg, operand) =>
        state.set(reg, state.get(reg) * state.get_value(operand)),
      Self::Divide(reg, operand) => {
        let right = state.get_value(operand);
        state.set(reg, state.get(reg) / right)
      },
      Self::Modulo(reg, operand) => {
        let left = state.get(reg);
        let right = state.get_value(operand);
        state.set(reg, left % right)
      },
      Self::Equal(reg, operand) =>
        state.set(reg, if state.get(reg) == state.get_value(operand) {1} else {0}),
    }
  }

  fn is_input(&self) -> bool {
    match self {
      Self::Input(_) => true,
      _ => false,
    }
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
  fn set(&mut self, reg: &Register, value: i64) -> i64{
    match reg {
      Register::W => self.w = value,
      Register::X => self.x = value,
      Register::Y => self.y = value,
      Register::Z => self.z = value,
    }
    value
  }

  fn get(&self, reg: &Register) -> i64 {
    match reg {
      Register::W => self.w,
      Register::X => self.x,
      Register::Y => self.y,
      Register::Z => self.z,
    }
  }

  fn get_value(&self, operand: &Operand) -> i64 {
    match operand {
      Operand::Register(reg) => self.get(reg),
      Operand::Value(val) => *val,
    }
  }

  fn find_input(&self, program: &[Operation], target: &[Option<i64>]) -> Option<Vec<i64>> {
    println!("Program {}: {:?}", program.len(), program.first().unwrap());
    'input: for input_value in (1..=9).rev() {
      println!("At {} Trying {}", program.len(), input_value);
      let mut state = self.clone();
      let mut pc = 0;
      while pc < program.len() && (pc == 0 || !program[pc].is_input()) {
        let val = program[pc].evaluate(&mut state, input_value);
        match target[pc] {
          Some(tgt) => if tgt != val {
            continue 'input
          },
          _ => {},
        }
        pc += 1;
      }
      if pc == program.len() {
        return Some(vec![input_value]);
      } else {
       match state.find_input(&program[pc..], &target[pc..]) {
         Some(answer) => {
           let mut result = answer.clone();
           result.push(input_value);
           return Some(result);
         }
         _ => {},
       }
      }
    }
    None
  }
}

#[derive(Clone, Debug)]
enum SymbolicExpression {
  Input(i64),
  Literal(i64),
  Operation(Rc<RefCell<SymbolicOperation>>),
}

impl SymbolicExpression {
  fn get_literal(&self) -> Option<i64> {
    match self {
      Self::Literal(v) => Some(*v),
      _ => None,
    }
  }

  fn get_bound(&self) -> SymbolicValue {
    match self {
      Self::Input(_) => SymbolicValue::from_range(1, 9),
      Self::Literal(v) => SymbolicValue::from_literal(*v),
      Self::Operation(x) => x.borrow().bounds.clone(),
    }
  }

  fn name(&self) -> String {
    match self {
      Self::Input(i) => format!("in_{}", i),
      Self::Literal(v) => format!("{}", v),
      Self::Operation(o) => format!("tmp_{:03}", o.borrow().name),
    }
  }

  fn print_operations(&self) {
    let mut done: Vec<usize> = Vec::new();
    match self {
      Self::Operation(o) => o.borrow().print_operations(&mut done),
      _ => {},
    }
  }

  fn propagate_back(&self, val: &SymbolicValue) {
    match self {
      Self::Operation(o) => o.borrow_mut().propagate_back(val),
      _ => {},
    }
  }

  fn find_operations(&self, operations:&mut Vec<Option<Rc<RefCell<SymbolicOperation>>>>) {
    match self {
      Self::Operation(o) => {
        let op = o.borrow();
        if operations[op.name].is_none() {
          operations[op.name] = Some(o.clone());
          op.left.find_operations(operations);
          op.right.find_operations(operations);
        }
      },
      _ => {},
    }
  }
}

impl fmt::Display for SymbolicExpression {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ValueRange {
  lower: i64,
  upper: i64,
}

impl fmt::Display for ValueRange {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.lower == self.upper {
      write!(f, "{}", self.lower)
    } else {
      write!(f, "{}..{}", self.lower, self.upper)
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SymbolicValue {
  ranges: Vec<ValueRange>,
}

impl SymbolicValue {
  fn from_literal(literal: i64) -> Self {
    Self::from_range(literal, literal)
  }

  fn from_range(lower: i64, upper: i64) -> Self {
    let mut ranges: Vec<ValueRange> = Vec::new();
    ranges.push(ValueRange{lower, upper});
    SymbolicValue{ranges}
  }

  fn from_set(set: &BTreeSet<i64>) -> Self {
    let mut ranges: Vec<ValueRange> = Vec::new();
    for val in set.iter() {
      if ranges.is_empty() || ranges.last().unwrap().upper != val - 1 {
        ranges.push(ValueRange{lower: *val, upper: *val});
      } else {
        ranges.last_mut().unwrap().upper = *val;
      }
    }
    SymbolicValue{ranges}
  }

  fn count(&self) -> usize {
    self.ranges.iter().map(|r| (r.upper - r.lower + 1) as usize)
        .fold(0, |a, b| a + b)
  }

  fn get_values(&self) -> SymbolicRangeIterator {
    SymbolicRangeIterator{val: &self, next_range: 0, next_elem: 0}
  }

  fn propagate<F>(&self, other: &SymbolicValue, func: F) -> SymbolicValue
      where F: Fn(i64, i64) -> Option<i64> {
    let mut result: BTreeSet<i64> = BTreeSet::new();
    for left in self.get_values() {
      for right in other.get_values() {
        match func(left, right) {
          None => false,
          Some(val) => result.insert(val),
        };
      }
    }
    Self::from_set(&result)
  }

  // Given a set of potential values for the left, right, and answer, find the inputs
  // that give one of the desired answers.
  fn propagate_back<F>(left: &SymbolicValue, right: &SymbolicValue,
                       answer: &SymbolicValue, func: F) -> (SymbolicValue, SymbolicValue)
      where F: Fn(i64, i64) -> Option<i64> {
    let mut left_result: BTreeSet<i64> = BTreeSet::new();
    let mut right_result: BTreeSet<i64> = BTreeSet::new();
    for left_value in left.get_values() {
      for right_value in right.get_values() {
        match func(left_value, right_value) {
          None => { },
          Some(val) => if answer.contains(val) {
            left_result.insert(left_value);
            right_result.insert(right_value);
          }
        };
      }
    }
    (Self::from_set(&left_result), Self::from_set(&right_result))
  }

  fn contains(&self, value: i64) -> bool {
    for rng in &self.ranges {
      if value >= rng.lower && value <= rng.upper {
        return true;
      }
    }
    false
  }

  fn is_disjoint(&self, other: &SymbolicValue) -> bool {
    let mut left_idx: usize = 0;
    let mut right_idx: usize = 0;
    while left_idx < self.ranges.len() && right_idx < other.ranges.len() {
      let left = &self.ranges[left_idx];
      let right = & other.ranges[right_idx];
      if left.lower <= right.lower {
        if right.lower <= left.upper {
          return false;
        }
        left_idx += 1;
      } else {
        if left.lower <= right.upper {
          return false;
        }
        right_idx += 1;
      }
    }
    true
  }
}

impl fmt::Display for SymbolicValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.ranges.len() < 10 {
      let parts: Vec<String> = self.ranges.iter().map(|r| format!("{}", r)).collect();
      write!(f, "{{ {} ({})}}", parts.join(", "), self.count())
    } else {
      write!(f, "{{ {}...{} ({})}}", self.ranges.first().unwrap().lower,
             self.ranges.last().unwrap().upper, self.count())
    }
  }
}

#[derive(Debug)]
struct SymbolicRangeIterator<'base> {
  val: &'base SymbolicValue,
  next_range: usize,
  next_elem: usize,
}

impl<'base> Iterator for SymbolicRangeIterator<'base> {
  type Item = i64;

  fn next(&mut self) -> Option<Self::Item> {
    while self.next_range < self.val.ranges.len() {
      let cur_range = &self.val.ranges[self.next_range];
      let val = self.next_elem as i64 + cur_range.lower;
      if val <= cur_range.upper {
        self.next_elem += 1;
        return Some(val);
      } else {
        self.next_range += 1;
        self.next_elem = 0;
      }
    }
    None
  }
}

#[derive(Clone, Debug)]
struct SymbolicOperation {
  name: usize,
  left: SymbolicExpression,
  right: SymbolicExpression,
  kind: SymbolicOperationKind,
  bounds: SymbolicValue,
}

impl SymbolicOperation {
  // If the result is a constant, compute it.
  fn literal_folding(&self) -> Option<i64> {
    match self.kind {
      SymbolicOperationKind::Add =>
        Some(self.left.get_literal()? + self.right.get_literal()?),
      SymbolicOperationKind::Multiply => {
        let left_value = self.left.get_literal();
        if let Some(l) = left_value {
          if l == 0 {
            return Some(0)
          }
        }
        let right_value = self.right.get_literal();
        if let Some(r) = right_value {
          if r == 0 {
            return Some(0)
          }
        }
        Some(left_value? * right_value?)
      }
      SymbolicOperationKind::Divide => {
        let left_value = self.left.get_literal();
        if let Some(l) = left_value {
          if l == 0 {
            return Some(0)
          }
        }
        let right_value = self.right.get_literal()?;
        if right_value == 0 {
          None
        } else {
          Some(left_value? / right_value)
        }
      }
      SymbolicOperationKind::Modulo => {
        let left_value = self.left.get_literal();
        if let Some(l) = left_value {
          if l == 0 {
            return Some(0)
          } else if l < 0 {
            return None
          }
        }
        let right_value = self.right.get_literal()?;
        if right_value <= 0 {
          None
        } else {
          Some(left_value? % right_value)
        }
      },
      SymbolicOperationKind::Equal => {
        let left_bounds = self.left.get_bound();
        let right_bounds = self.right.get_bound();
        if left_bounds.is_disjoint(&right_bounds) {
          // no overlap in the ranges
          Some(0)
        } else if left_bounds.count() == 1 && right_bounds.count() == 1 &&
            left_bounds == right_bounds {
          // one potential value on each side
          Some(1)
        } else {
          // we can't tell
          None
        }
      }
    }
  }

  // Handle the cases where we don't need the operation, because it is an identity.
  // (eg. x + 0 or x * 1)
  fn reduction(&self) -> Option<SymbolicExpression> {
    match self.kind {
      SymbolicOperationKind::Add => {
        let left_value = self.left.get_literal();
        if let Some(l) = left_value {
          if l == 0 {
            return Some(self.right.clone())
          }
        }
        let right_value = self.right.get_literal();
        if let Some(r) = right_value {
          if r == 0 {
            return Some(self.left.clone())
          }
        }
        None
      }
      SymbolicOperationKind::Multiply => {
        let left_value = self.left.get_literal();
        if let Some(l) = left_value {
          if l == 1 {
            return Some(self.right.clone())
          }
        }
        let right_value = self.right.get_literal();
        if let Some(r) = right_value {
          if r == 1 {
            return Some(self.left.clone())
          }
        }
        None
      }
      SymbolicOperationKind::Divide => {
        let right_value = self.right.get_literal()?;
        if right_value == 1 {
          return Some(self.left.clone());
        }
        None
      }
      _ => None,
    }
  }

  fn print_operations(&self, done: &mut Vec<usize>) {
    if !done.contains(&self.name) {
      done.push(self.name);
      match &self.left {
        SymbolicExpression::Operation(op) =>
          op.borrow().print_operations(done),
        _ => {}
      }
      match &self.right {
        SymbolicExpression::Operation(op) =>
          op.borrow().print_operations(done),
        _ => {}
      }
      let left_bounds = self.left.get_bound();
      let right_bounds = self.right.get_bound();
      println!("tmp_{:03} <- {} {} {} [{} {} {} = {}]", self.name, self.left, self.kind, self.right,
               left_bounds, self.kind, right_bounds, self.bounds);
    }
  }

  fn propagate_back(&mut self, val: &SymbolicValue) {
    if self.bounds.count() != val.count() {
      self.bounds = val.clone();
      let (left, right) = SymbolicValue::propagate_back(
        &self.left.get_bound(),
        &self.right.get_bound(),
        &self.bounds,
        self.kind.operation());
      self.left.propagate_back(&left);
      self.right.propagate_back(&right);
    }
  }
}

#[derive(Clone, Copy, Debug)]
enum SymbolicOperationKind {
  Add,
  Multiply,
  Divide,
  Modulo,
  Equal,
}

impl SymbolicOperationKind {
  fn operation(&self) -> impl Fn(i64, i64) -> Option<i64> {
    match self {
      Self::Add => move |l, r| Some (l+r),
      Self::Multiply => move |l, r| Some (l*r),
      Self::Divide => move |l, r| {
        if r == 0 {
          None
        } else {
          Some(l/r)
        }
      },
      Self::Modulo => move |l, r| {
        if r <= 0 {
          None
        } else {
          Some(l % r)
        }
      },
      Self::Equal => move |l, r| Some(if l == r {1} else {0}),
    }
  }
}

impl fmt::Display for SymbolicOperationKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      SymbolicOperationKind::Add => write!(f, "+"),
      SymbolicOperationKind::Multiply => write!(f, "*"),
      SymbolicOperationKind::Divide => write!(f, "/"),
      SymbolicOperationKind::Modulo => write!(f, "%"),
      SymbolicOperationKind::Equal => write!(f, "=="),
    }
  }
}

#[derive(Clone, Debug)]
struct SymbolicState {
  w: SymbolicExpression,
  x: SymbolicExpression,
  y: SymbolicExpression,
  z: SymbolicExpression,
}

impl SymbolicState {
  fn new() -> Self {
    SymbolicState{
      w: SymbolicExpression::Literal(0),
      x: SymbolicExpression::Literal(0),
      y: SymbolicExpression::Literal(0),
      z: SymbolicExpression::Literal(0),
    }
  }

  fn set(&mut self, reg: &Register, value: SymbolicExpression) {
    match reg {
      Register::W => self.w = value,
      Register::X => self.x = value,
      Register::Y => self.y = value,
      Register::Z => self.z = value,
    }
  }

  fn get(&self, reg: &Register) -> SymbolicExpression {
    match reg {
      Register::W => self.w.clone(),
      Register::X => self.x.clone(),
      Register::Y => self.y.clone(),
      Register::Z => self.z.clone(),
    }
  }

  fn get_value(&self, operand: &Operand) -> SymbolicExpression {
    match operand {
      Operand::Value(val) => SymbolicExpression::Literal(*val),
      Operand::Register(reg) => self.get(reg),
    }
  }

  fn make_value(name: usize,
                kind: SymbolicOperationKind,
                left: SymbolicExpression,
                right: SymbolicExpression,
                ) -> SymbolicExpression {
    let bounds = left.get_bound().propagate(&right.get_bound(),
                                            kind.operation());
    let op = SymbolicOperation {name, kind, left, right, bounds};
    if let Some(answer) = op.literal_folding() {
      SymbolicExpression::Literal(answer)
    } else if let Some(simplified) = op.reduction() {
      simplified
    } else {
      SymbolicExpression::Operation(Rc::new(RefCell::new(op)))
    }
  }

  fn interpret_operation(&mut self, op: &Operation, operation_idx: &mut usize, input_idx: &mut i64) {
    match op {
      Operation::Input(reg) => {
        self.set(reg, SymbolicExpression::Input(*input_idx));
        *input_idx += 1;
      }
      Operation::Add(reg, operand) =>
        self.set(reg, Self::make_value(*operation_idx,
                                       SymbolicOperationKind::Add,
                                       self.get(reg),
                                        self.get_value(operand))),
      Operation::Multiply(reg, operand) =>
        self.set(reg, Self::make_value(*operation_idx,
                                       SymbolicOperationKind::Multiply,
                                       self.get(reg),
                                       self.get_value(operand))),
      Operation::Divide(reg, operand) =>
        self.set(reg, Self::make_value(*operation_idx,
                                       SymbolicOperationKind::Divide,
                                       self.get(reg),
                                       self.get_value(operand))),
      Operation::Modulo(reg, operand) =>
        self.set(reg, Self::make_value(*operation_idx,
                                       SymbolicOperationKind::Modulo,
                                       self.get(reg),
                                       self.get_value(operand))),
      Operation::Equal(reg, operand) =>
        self.set(reg, Self::make_value(*operation_idx,
                                       SymbolicOperationKind::Equal,
                                       self.get(reg),
                                       self.get_value(operand))),
    }
    *operation_idx += 1;
  }

  fn interpret(program: &Vec<Operation>) -> Self {
    let mut state = Self::new();
    let mut operation_idx: usize = 0;
    let mut input_idx = 0;
    for op in program {
      state.interpret_operation(op, &mut operation_idx, &mut input_idx);
    }
    state
  }
}

impl fmt::Display for SymbolicState {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "w: {} {}", self.w, self.w.get_bound().count())?;
    writeln!(f, "x: {} {}", self.x, self.x.get_bound().count())?;
    writeln!(f, "y: {} {}", self.y, self.y.get_bound().count())?;
    writeln!(f, "z: {} {}", self.z, self.z.get_bound().count())
  }
}

fn main() {
  let stdin = io::stdin();
  let program: Vec<Operation> = stdin.lock().lines()
      .map(|x| String::from(x.unwrap()))
      .filter(|x| x.len() > 0)
      .map(|x| Operation::parse(&x).unwrap())
      .collect();

  let symbolic = SymbolicState::interpret(&program);
  symbolic.z.print_operations();
  println!("z = {}", symbolic.z);
  println!();

  symbolic.z.propagate_back(&SymbolicValue::from_literal(0));
  symbolic.z.print_operations();

  // Get the list of operations that impact the final z value
  let mut operations: Vec<Option<Rc<RefCell<SymbolicOperation>>>> = vec![None; program.len()];
  symbolic.z.find_operations(&mut operations);
  let mut needed: Vec<bool> = vec![false; program.len()];
  let mut target: Vec<Option<i64>> = vec![None; program.len()];
  for i in 0..program.len() {
    // If the operation has a single valid value, record it as a target.
    match &operations[i] {
      Some(op) => {
        let bound = &op.borrow().bounds;
        if bound.count() == 1 {
          target[i] = Some(bound.ranges.first().unwrap().lower);
          println!("operation {}: {:?} = {}", i, program[i], target[i].unwrap());
        }
      }
      _ => {}
    }
    // Record whether we need to execute that instruction
    needed[i] = operations[i].is_some() || program[i].is_input();
  }
  let result = State::default().find_input(&program, &target);
  println!("result: {:?}", result);
}
