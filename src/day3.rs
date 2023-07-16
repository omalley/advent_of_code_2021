fn compute_width(inputs: &Vec<u64>) -> u32 {
  let mask: u64 = inputs.iter().fold(0, |x, y| x | y);
  64 - u64::leading_zeros(mask)
}

// Compute the difference in the count of ones versus zeros at
// the given mask position.
fn compare_bits(inputs: &Vec<u64>, mask: u64) -> i32 {
  let mut result = 0;
  for val in inputs {
    if val & mask == 0 {
      result -= 1;
    } else {
      result += 1;
    }
  }
  result
}

// req_bit takes the difference in 1's versus 0's and returns the
// required value for the bit.
fn compute_rating<F>(inputs: &Vec<u64>, mask: u64, req_bit: F) -> u64
    where F: Fn(i32) -> bool {

  // if we have no mask or inputs, something went wrong
  assert!(mask != 0 && inputs.len() > 0);

  // determine whether we need a 0 or 1 for this pass
  let required_bit = req_bit(compare_bits(&inputs, mask));

  // filter the numbers with the right value at the mask position
  let sub_list: Vec<u64> =
    inputs.iter().filter(|x| ((*x & mask) != 0) == required_bit)
          .map(|x| *x).collect();

  // if we have a single answer use it, otherwise continue
  if sub_list.len() == 1 {
    sub_list[0]
  } else {
    compute_rating(&sub_list, mask >> 1, req_bit)
  }
}

pub fn generator(data: &str)-> Vec<u64> {
  data.lines()
    .map(|x| u64::from_str_radix(x.trim(), 2)
      .unwrap()).collect()
}

pub fn part1(inputs: &Vec<u64>) -> u64 {
  let width = compute_width(inputs);
  let mut bit_mask: u64 = 1 << (width - 1);
  let mut gamma: u64 = 0;
  while bit_mask != 0 {
    gamma <<= 1;
    if compare_bits(inputs, bit_mask) > 0 {
      gamma |= 1;
    }
    bit_mask >>= 1;
  }
  // invert gamma to get epsilon
  let epsilon = (!gamma) & ((1 << width) - 1);
  gamma * epsilon
}

pub fn part2(inputs: &Vec<u64>) -> u64 {
  let mask = 1 << (compute_width(inputs) - 1);
  let o2_rating = compute_rating(inputs, mask, |x| x >= 0);
  let co2_rating = compute_rating(inputs, mask, |x| x < 0);
  o2_rating * co2_rating
}
