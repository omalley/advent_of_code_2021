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
enum SymbolicValue {
  Input(i64),
  Literal(i64),
  Operation(Rc<RefCell<SymbolicOperation>>),
}

impl SymbolicValue {
  fn get_literal(&self) -> Option<i64> {
    match self {
      Self::Literal(v) => Some(*v),
      _ => None,
    }
  }

  fn get_bound(&self) -> (i64, i64) {
    match self {
      Self::Input(_) => (1, 9),
      Self::Literal(v) => (*v, *v),
      Self::Operation(x) => x.borrow().bounds,
    }
  }

  fn name(&self) -> String {
    match self {
      Self::Input(i) => format!("in_{}", i),
      Self::Literal(v) => format!("{}", v),
      Self::Operation(o) => format!("tmp_{:03}", o.borrow().name),
    }
  }

  fn set_require(&self, bounds: Option<(i64, i64)>) {
    match self {
      Self::Operation(op) => op.borrow_mut().set_require(bounds),
      _ => {},
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

impl fmt::Display for SymbolicValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}

#[derive(Clone, Debug)]
struct SymbolicOperation {
  name: usize,
  left: SymbolicValue,
  right: SymbolicValue,
  kind: SymbolicOperationKind,
  bounds: (i64, i64),
  require: Option<(i64, i64)>,
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
        if left_bounds.1 < right_bounds.0 || right_bounds.1 < left_bounds.0 {
          // no overlap in the ranges
          Some(0)
        } else if left_bounds.0 == left_bounds.1 && left_bounds.0 == right_bounds.0 &&
            left_bounds.0 == right_bounds.1 {
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
  fn reduction(&self) -> Option<SymbolicValue> {
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
        SymbolicValue::Operation(op) =>
          op.borrow().print_operations(done),
        _ => {}
      }
      match &self.right {
        SymbolicValue::Operation(op) =>
          op.borrow().print_operations(done),
        _ => {}
      }
      let left_bounds = self.left.get_bound();
      let right_bounds = self.right.get_bound();
      print!("tmp_{:03} <- {} {} {} [{}..{} {} {}..{} = {}..{}]", self.name, self.left, self.kind, self.right,
               left_bounds.0, left_bounds.1, self.kind, right_bounds.0, right_bounds.1, self.bounds.0, self.bounds.1);
      if let Some(req) = self.require {
        print!(" req: {}..{}", req.0, req.1);
      }
      println!();
    }
  }

  fn propagate_multiply(&self, require:(i64, i64), other:(i64, i64)) -> Option<(i64,i64)> {
    // if other can be zero, we don't know anything
    if other.0 == 0 || other.1 == 0 || (other.0 > 0) != (other.1 > 0) {
      None
    } else if require.0 >= 0 {
      // if we end up with a natural number
      if other.0 > 0 {
        Some((require.0/other.0, require.1/other.1))
      } else {
        Some((require.1/other.0, require.0/other.1))
      }
    } else if require.1 < 0 {
      // if we end up with a negative number
      if other.0 > 0 {
        Some((require.0/other.1, require.1/other.0))
      } else {
        Some((require.1/other.1, require.0/other.0))
      }
    } else {
      // or it can be either positive or negative
      if other.0 > 0 {
        Some((require.0/other.0, require.1/other.0))
      } else {
        Some((require.1/other.1, require.0/other.1))
      }
    }
  }

  fn set_require(&mut self, require: Option<(i64, i64)>) {
    if require.is_none() {
      self.require = None
    } else {
      let require = require.unwrap();
      self.require = Some((i64::min(self.bounds.1, i64::max(require.0, self.bounds.0)),
                           i64::max(self.bounds.0, i64::min(require.1, self.bounds.1))));
      let require = self.require.unwrap();
      let left_bound = self.left.get_bound();
      let right_bound = self.right.get_bound();
      match self.kind {
        SymbolicOperationKind::Add => {
          self.left.set_require(Some((require.0 - right_bound.1, require.1 - right_bound.0)));
          self.right.set_require(Some((require.0 - left_bound.1, require.1 - left_bound.0)));
        }
        SymbolicOperationKind::Multiply => {
          self.right.set_require(self.propagate_multiply(require, left_bound));
          self.left.set_require(self.propagate_multiply(require, right_bound));
        }
        SymbolicOperationKind::Divide => {
          if (0, 0) == self.require.unwrap() {
            if right_bound.0 > 0 {
              self.left.set_require(Some((0, right_bound.0 - 1)));
            }
          }
        }
        SymbolicOperationKind::Equal => {
          // Do we have a fixed answer that we need?
          if require.0 == require.1 {
            if require.0 == 1 {
              // For true, set the bounds of each side to the other side.
              self.left.set_require(Some(self.right.get_bound()));
              self.right.set_require(Some(self.left.get_bound()));
            } else if right_bound.0 == right_bound.1 && left_bound.0 == right_bound.0 {
              self.left.set_require(Some((left_bound.0 + 1, left_bound.1)));
            }
          }
        }
        _ => {},
      }

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
  w: SymbolicValue,
  x: SymbolicValue,
  y: SymbolicValue,
  z: SymbolicValue,
}

impl SymbolicState {
  fn new() -> Self {
    SymbolicState{
      w: SymbolicValue::Literal(0),
      x: SymbolicValue::Literal(0),
      y: SymbolicValue::Literal(0),
      z: SymbolicValue::Literal(0),
    }
  }

  fn set(&mut self, reg: &Register, value: SymbolicValue) {
    match reg {
      Register::W => self.w = value,
      Register::X => self.x = value,
      Register::Y => self.y = value,
      Register::Z => self.z = value,
    }
  }

  fn get(&self, reg: &Register) -> SymbolicValue {
    match reg {
      Register::W => self.w.clone(),
      Register::X => self.x.clone(),
      Register::Y => self.y.clone(),
      Register::Z => self.z.clone(),
    }
  }

  fn get_value(&self, operand: &Operand) -> SymbolicValue {
    match operand {
      Operand::Value(val) => SymbolicValue::Literal(*val),
      Operand::Register(reg) => self.get(reg),
    }
  }

  fn compute_bounds(kind:SymbolicOperationKind,
                    left: (i64, i64), right: (i64, i64)) -> (i64, i64) {
    match kind {
      SymbolicOperationKind::Add => (left.0 + right.0, left.1 + right.1),
      SymbolicOperationKind::Multiply => (
        i64::min(i64::min(left.0 * right.1, left.1 * right.0),
                 i64::min(left.0 * right.0, left.1 * right.1)),
        i64::max(i64::max(left.0 * right.1, left.1 * right.0),
                 i64::max(left.0 * right.0, left.1 * right.1))
      ),
      SymbolicOperationKind::Divide => {
        let right_lower = if right.0 == 0 { 1 } else { right.0 };
        let right_upper = if right.1 == 0 { -1 } else { right.1 };
        (i64::min(i64::min(left.0 / right_lower, left.1 / right_lower),
                  i64::min(left.0 / right_upper, left.1 / right_upper)),
         i64::max(i64::max(left.0 / right_lower, left.1 / right_lower),
                  i64::max(left.0 / right_upper, left.1 / right_upper)))
      }
      SymbolicOperationKind::Modulo => (0, right.1 - 1),
      SymbolicOperationKind::Equal => (0, 1),
    }
  }

  fn make_value(name: usize,
                kind: SymbolicOperationKind,
                left: SymbolicValue,
                right: SymbolicValue,
                ) -> SymbolicValue {
    let bounds = Self::compute_bounds(kind, left.get_bound(), right.get_bound());
    let op = SymbolicOperation {name, kind, left, right, bounds, require: None};
    if let Some(answer) = op.literal_folding() {
      SymbolicValue::Literal(answer)
    } else if let Some(simplified) = op.reduction() {
      simplified
    } else {
      SymbolicValue::Operation(Rc::new(RefCell::new(op)))
    }
  }

  fn interpret_operation(&mut self, op: &Operation, operation_idx: &mut usize, input_idx: &mut i64) {
    match op {
      Operation::Input(reg) => {
        self.set(reg, SymbolicValue::Input(*input_idx));
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

fn main() {
  let stdin = io::stdin();
  let operators: Vec<Operation> = stdin.lock().lines()
      .map(|x| String::from(x.unwrap()))
      .filter(|x| x.len() > 0)
      .map(|x| Operation::parse(&x).unwrap())
      .collect();

  let symbolic = SymbolicState::interpret(&operators);
  symbolic.z.set_require(Some((0,0)));
  symbolic.z.print_operations();
  println!("z = {}", symbolic.z);
}
