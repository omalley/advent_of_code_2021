use std::cell::RefCell;
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
  fn evaluate(&self, state: &mut State, input: i64) -> bool {
    match self {
      Self::Input(reg) => {
        state.set(reg, input);
      },
      Self::Add(reg, operand) =>
        state.set(reg, state.get(reg) + state.get_value(operand)),
      Self::Multiply(reg, operand) =>
        state.set(reg, state.get(reg) * state.get_value(operand)),
      Self::Divide(reg, operand) => {
        let right = state.get_value(operand);
        if right == 0 {
          return false;
        }
        state.set(reg, state.get(reg) / right)
      },
      Self::Modulo(reg, operand) => {
        let left = state.get(reg);
        let right = state.get_value(operand);
        if left < 0 || right <= 0 {
          return false;
        }
        state.set(reg, left % right)
      },
      Self::Equal(reg, operand) =>
        state.set(reg, if state.get(reg) == state.get_value(operand) {1} else {0}),
    }
    return true
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
  fn set(&mut self, reg: &Register, value: i64) {
    match reg {
      Register::W => self.w = value,
      Register::X => self.x = value,
      Register::Y => self.y = value,
      Register::Z => self.z = value,
    }
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
}

#[derive(Debug, Default)]
struct CodeBlock {
  block: Vec<Operation>,
}

impl CodeBlock {
  fn evaluate(&self, state: &mut State, input: i64) -> bool {
    for op in &self.block {
      if !op.evaluate(state, input) {
        return false;
      }
    }
    true
  }
}

// Destructively splits a list of operations into blocks where each block
// starts with an input.
fn split_blocks(ops: &mut Vec<Operation>) -> Vec<CodeBlock> {
  let mut result: Vec<CodeBlock> = Vec::new();
  let mut current = CodeBlock::default();
  while !ops.is_empty() {
    if let Some(Operation::Input(_)) = ops.get(0) {
      if current.block.len() > 0 {
        result.push(current);
        current = CodeBlock::default();
      }
    }
    current.block.push(ops.remove(0));
  }
  if current.block.len() > 0 {
    result.push(current);
  }
  result
}

fn find_model(blocks: &[CodeBlock], state: &State) -> Option<Vec<i64>> {
  for next_input in (1..=9).rev() {
    let mut next_state = state.clone();
    if blocks[0].evaluate(&mut next_state, next_input) {
      if blocks.len() > 1 {
        if let Some(answer) = find_model(&blocks[1..], &next_state) {
          let mut result: Vec<i64> = Vec::new();
          result.push(next_input);
          for a in answer {
            result.push(a)
          }
        }
      } else if next_state.z == 0 {
        return Some(vec![next_input])
      }
    }
  }
  None
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

  // sort the ranges and remove duplicates
  fn normalize(&mut self) {
    if self.ranges.len() > 1 {
      println!("start sort");
      self.ranges.sort();
      self.ranges.dedup();
      println!("end sort");
      let mut last = self.ranges[0].upper;
      let mut idx: usize = 1;
      while idx < self.ranges.len() {
        println!("idx = {}/{}, last = {}", idx, self.ranges.len(), last);
        if self.ranges[idx].lower <= last {
          last = i64::max(last, self.ranges[idx].upper);
          self.ranges[idx - 1].upper = last;
          self.ranges.remove(idx);
        } else {
          idx += 1;
        }
      }
    }
  }

  fn count(&self) -> usize {
    self.ranges.iter().map(|r| (r.upper - r.lower + 1) as usize)
        .fold(0, |a, b| a + b)
  }

  fn get_values(&self) -> SymbolicRangeIterator {
    SymbolicRangeIterator{val: &self, next_range: 0, next_elem: 0}
  }

  fn add(&self, other: &SymbolicValue) -> SymbolicValue {
    let mut result = SymbolicValue{ranges: Vec::new() };
    for left in &self.ranges {
      for right in &other.ranges {
        result.ranges.push(ValueRange{lower: left.lower + right.lower,
                                            upper: left.upper + right.upper});
      }
    }
    result.normalize();
    result
  }

  fn multiply(&self, other: &SymbolicValue) -> SymbolicValue {
    let mut result = SymbolicValue{ranges: Vec::new() };
    for left in self.get_values() {
      for right in other.get_values() {
        let prod = left * right;
        result.ranges.push(ValueRange{lower: prod, upper: prod});
      }
    }
    result.normalize();
    result
  }

  fn divide(&self, other: &SymbolicValue) -> SymbolicValue {
    let mut result = SymbolicValue{ranges: Vec::new() };
    for left in self.get_values() {
      for right in other.get_values() {
        if right != 0 {
          let ans = left / right;
          result.ranges.push(ValueRange { lower: ans, upper: ans });
        }
      }
    }
    result.normalize();
    result
  }

  fn modulo(&self, other: &SymbolicValue) -> SymbolicValue {
    let mut result = SymbolicValue{ranges: Vec::new() };
    for left in self.get_values() {
      for right in other.get_values() {
        if right > 0 {
          let ans = left % right;
          result.ranges.push(ValueRange { lower: ans, upper: ans });
        }
      }
    }
    println!("start normalize with {}", result.ranges.len());
    result.normalize();
    println!("end normalize with {}", result.ranges.len());
    result
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
    let parts: Vec<String> = self.ranges.iter().map(|r| format!("{}", r)).collect();
    write!(f, "{{ {} }}", parts.join(","))
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

#[derive(Clone, Copy, Debug)]
enum SymbolicOperationKind {
  Add,
  Multiply,
  Divide,
  Modulo,
  Equal,
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

  fn compute_bounds(kind:SymbolicOperationKind,
                    left: &SymbolicValue, right: &SymbolicValue) -> SymbolicValue {
    match kind {
      SymbolicOperationKind::Add => left.add(&right),
      SymbolicOperationKind::Multiply => left.multiply(&right),
      SymbolicOperationKind::Divide => left.divide(&right),
      SymbolicOperationKind::Modulo => left.modulo(&right),
      SymbolicOperationKind::Equal => SymbolicValue::from_range(0, 1),
    }
  }

  fn make_value(name: usize,
                kind: SymbolicOperationKind,
                left: SymbolicExpression,
                right: SymbolicExpression,
                ) -> SymbolicExpression {
    let bounds = Self::compute_bounds(kind, &left.get_bound(), &right.get_bound());
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
    println!("starting {}: {:?}", *operation_idx, op);
    println!("{}", self);
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
    writeln!(f, "w: {} {}", self.w, self.w.get_bound().count());
    writeln!(f, "x: {} {}", self.x, self.x.get_bound().count());
    writeln!(f, "y: {} {}", self.y, self.y.get_bound().count());
    writeln!(f, "z: {} {}", self.z, self.z.get_bound().count())
  }
}

fn main() {
  let stdin = io::stdin();
  let operators: Vec<Operation> = stdin.lock().lines()
      .map(|x| String::from(x.unwrap()))
      .filter(|x| x.len() > 0)
      .map(|x| Operation::parse(&x).unwrap())
      .collect();

  let symbolic = SymbolicState::interpret(&operators);
  symbolic.z.print_operations();
  println!("z = {}", symbolic.z);
}
