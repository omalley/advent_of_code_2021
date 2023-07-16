fn count_descents(nums: &Vec<i64>) -> usize {
  let mut count = 0;
  let mut last = i64::MAX;
  for current in nums {
    if *current > last {
      count += 1;
    }
    last = *current;
  }
  count
}

fn count_triple_descents(nums: &Vec<i64>) -> usize {
  let mut count = 0;
  let mut last = i64::MAX;
  for i in 0..nums.len()-2 {
    let current = nums[i] + nums[i+1] + nums[i+2];
    if current > last {
      count += 1;
    }
    last = current;
  }
  count
}

pub fn generator(data: &str) -> Vec<i64> {
  data.lines().map(|x| x.trim().parse::<i64>().unwrap()).collect()
}

pub fn part1(input: &Vec<i64>) -> usize {
  count_descents(input)
}

pub fn part2(input: &Vec<i64>) -> usize {
  count_triple_descents(input)
}