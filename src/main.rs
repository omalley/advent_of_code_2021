use std::collections::BTreeMap;
use argh::FromArgs;
use colored::Colorize;
use omalley_aoc2021::{DayResult,FUNCS,INPUTS,NAMES,time};
use serde::{Deserialize,Serialize};

#[derive(FromArgs)]
/** Solution for Advent of Code (https://adventofcode.com/)*/
struct Args {
  /// A single day to execute (all days by default)
  #[argh(option, short = 'd')]
  day: Option<usize>,
}

#[derive(Default,Deserialize,Serialize)]
struct Answers {
  // map from day name to answers
  days: BTreeMap<String,Vec<String>>,
}

impl Answers {
  const FILENAME: &'static str = "answers.yml";

  fn read() -> Self {
    if let Ok(f) = std::fs::File::open(Self::FILENAME) {
      serde_yaml::from_reader(f).expect("Could not read answers")
    } else {
      Self::default()
    }
  }

  fn update(&mut self, delta_list: &Vec<DayResult>) {
    for delta in delta_list {
      let new_val = delta.get_answers();
      if let Some(prev) =
          self.days.insert(delta.day.to_string(), new_val.clone()) {
        if prev != new_val {
          println!("{}", format!("Output for {} changed from {:?} to {:?}!",
                                 delta.pretty_day(), prev, new_val).bold());
        }
      }
    }
  }

  fn write(&self) {
    let f = std::fs::OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open(Self::FILENAME)
      .expect("Couldn't open file");
    serde_yaml::to_writer(f, self).unwrap();
  }
}

fn main() {
    let args: Args = argh::from_env();
    // Did the user pick a single day to run
    let day_filter: Option<usize> = match args.day {
        Some(day) => {
            let name = format!("day{}", day);
            Some(NAMES.iter().position(|x| **x == name)
              .expect("Requested an unimplemented day"))
        },
        None => None
    };

     let (elapsed, results) = time(&|| {
        crate::FUNCS.iter().enumerate()
          .filter(|(p, _)| day_filter.is_none() || day_filter.unwrap() == *p)
          .map(|(p, f)| f(INPUTS[p]))
          .collect::<Vec<DayResult>>()
    });

    for r in &results {
      println!("{}", r);
    }
    println!("{} {}", "Overall runtime".bold(), format!("({:.2?})", elapsed).dimmed());

    let mut old_answers = Answers::read();
    old_answers.update(&results);
    old_answers.write();
}
