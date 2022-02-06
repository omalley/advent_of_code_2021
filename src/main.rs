use std::cell::RefCell;
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
}

#[derive(Clone, Debug)]
enum SymbolicOperation {
  Add(SymbolicValue, SymbolicValue),
  Multiply(SymbolicValue, SymbolicValue),
  Divide(SymbolicValue, SymbolicValue),
  Modulo(SymbolicValue, SymbolicValue),
  Equal(SymbolicValue, SymbolicValue),
}

impl SymbolicOperation {
  fn literal_folding(&self) -> Option<i64> {
    match self {
      Self::Add(left, right) =>
        Some(left.get_literal()? + right.get_literal()?),
      Self::Multiply(left, right) => {
        let left_value = left.get_literal();
        if let Some(l) = left_value {
          if l == 0 {
            return Some(0)
          }
        }
        let right_value = right.get_literal();
        if let Some(r) = right_value {
          if r == 0 {
            return Some(0)
          }
        }
        Some(left_value? * right_value?)
      }
      Self::Divide(left, right) => {
        let left_value = left.get_literal();
        if let Some(l) = left_value {
          if l == 0 {
            return Some(0)
          }
        }
        let right_value = right.get_literal()?;
        if right_value == 0 {
          None
        } else {
          Some(left_value? / right_value)
        }
      }
      Self::Modulo(left, right) => {
        let left_value = left.get_literal();
        if let Some(l) = left_value {
          if l == 0 {
            return Some(0)
          } else if l < 0 {
            return None
          }
        }
        let right_value = right.get_literal()?;
        if right_value <= 0 {
          None
        } else {
          Some(left_value? % right_value)
        }
      },
      Self::Equal(left, right) =>
        Some(if left.get_literal()? == right.get_literal()? { 1 } else { 0 }),
    }
  }

  fn reduction(&self) -> Option<SymbolicValue> {
    match self {
      Self::Add(left, right) => {
        let left_value = left.get_literal();
        if let Some(l) = left_value {
          if l == 0 {
            return Some(right.clone())
          }
        }
        let right_value = right.get_literal();
        if let Some(r) = right_value {
          if r == 0 {
            return Some(left.clone())
          }
        }
        None
      }
      Self::Multiply(left, right) => {
        let left_value = left.get_literal();
        if let Some(l) = left_value {
          if l == 1 {
            return Some(right.clone())
          }
        }
        let right_value = right.get_literal();
        if let Some(r) = right_value {
          if r == 1 {
            return Some(left.clone())
          }
        }
        None
      }
      Self::Divide(left, right) => {
        let right_value = right.get_literal()?;
        if right_value == 1 {
          return Some(left.clone());
        }
        None
      }
      _ => None,
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

  fn make_value(op: SymbolicOperation) -> SymbolicValue {
    if let Some(answer) = op.literal_folding() {
      SymbolicValue::Literal(answer)
    } else if let Some(simplified) = op.reduction() {
      simplified
    } else {
      SymbolicValue::Operation(Rc::new(RefCell::new(op)))
    }
  }

  fn do_operation(&mut self, op: &Operation, input_idx: &mut i64) {
    match op {
      Operation::Input(reg) => {
        self.set(reg, SymbolicValue::Input(*input_idx));
        *input_idx += 1;
      }
      Operation::Add(reg, operand) => {
        self.set(reg, Self::make_value(SymbolicOperation::Add(
          self.get(reg), self.get_value(operand))))
      }
      Operation::Multiply(reg, operand) => {
        self.set(reg, Self::make_value(SymbolicOperation::Multiply(
          self.get(reg), self.get_value(operand))))
      }
      Operation::Divide(reg, operand) => {
        self.set(reg, Self::make_value(SymbolicOperation::Divide(
          self.get(reg), self.get_value(operand))))
      }
      Operation::Modulo(reg, operand) => {
        self.set(reg, Self::make_value(SymbolicOperation::Modulo(
          self.get(reg), self.get_value(operand))))
      }
      Operation::Equal(reg, operand) => {
        self.set(reg, Self::make_value(SymbolicOperation::Equal(
          self.get(reg), self.get_value(operand))))
      }
    }
  }

  fn interpret(program: &Vec<Operation>) -> Self {
    let mut state = Self::new();
    let mut input_idx = 0;
    for op in program {
      state.do_operation(op, &mut input_idx);
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
  println!("output = {:?}", symbolic.z);
}
