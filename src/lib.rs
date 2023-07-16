use std::cmp::min;
use std::fmt;
use std::time;

use colored::Colorize;

/// Format the output of each line of the output.
/// Includes the category, time, and result.
fn pretty_print(f: &mut fmt::Formatter<'_>, line: &str,
                duration: time::Duration,
                output: Option<&str>) -> fmt::Result {
    const DISPLAY_WIDTH: usize = 40;

    let duration = format!("({:.2?})", duration);
    write!(f, "{} {}", line, duration.dimmed())?;

    match output {
        Some(output) => {
            let width = "  - ".len() + line.chars().count() + 1 + duration.chars().count();
            let dots = DISPLAY_WIDTH - min(DISPLAY_WIDTH - 5, width) - 2;
            write!(f, " {}", ".".repeat(dots).dimmed())?;

            if output.contains('\n') {
                writeln!(f)?;

                for line in output.trim_matches('\n').lines() {
                    writeln!(f, "    {}", line.bold())?;
                }
                Ok(())
            } else {
                writeln!(f, " {}", output.bold())
            }
        },
        None => writeln!(f),
    }
}

/// Time the given function, returning its result and the elapsed time
pub fn time<T>(func: &dyn Fn() -> T) -> (time::Duration, T) {
    let start = time::Instant::now();
    let result = func();

    (start.elapsed(), result)
}

/// The times and results of running a day's code.
pub struct DayResult {
    pub day: String,
    generate_time: time::Duration,
    part1: (time::Duration, String),
    part2: (time::Duration, String),
}

impl DayResult {
  /// Return the pretty name for the day
  pub fn pretty_day(&self) -> String {
    self.day.replace("day", "Day ")
  }

  /// Get the answers without the times
  pub fn get_answers(&self) -> Vec<String> {
    vec![self.part1.1.to_string(), self.part2.1.to_string()]
  }
}

impl fmt::Display for DayResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let duration = format!("({:.2?})", self.generate_time + self.part1.0 + self.part2.0);
        writeln!(f, "{} {}", self.pretty_day().bold(), duration.dimmed())?;
        pretty_print(f," · Generator", self.generate_time, None)?;
        pretty_print(f, " · Part 1", self.part1.0, Some(&self.part1.1))?;
        pretty_print(f, " · Part 2", self.part2.0, Some(&self.part2.1))
    }
}

#[macro_export]
macro_rules! day_list {
    ( $($day:ident),+ $(,)?) => {
        // Each day's code should be in src/day?.rs.
        $(pub mod $day;)+

        /// Build a lambda to run each day's code
        pub const FUNCS : &[&dyn Fn(&str) -> DayResult] = &[
            $(&|data| {
                let (generate_time, input) = time(&|| $day::generator(data));
                let part1 = time(&|| $day::part1(&input));
                let part2 = time(&|| $day::part2(&input));
                DayResult{day: stringify!($day).to_string(),
                          generate_time,
                          part1: (part1.0, part1.1.to_string()),
                          part2: (part2.0, part2.1.to_string())}},)+
        ];

        /// Define the list of implemented day names.
        pub const NAMES: &[&str] = &[$(stringify!($day)),*];

        /// Load the inputs for each day into an array.
        pub const INPUTS: &[&str] = &[$(include_str!(concat!("../input/", stringify!($day), ".txt"))),*];
}}

day_list!(
        day1,
        day2,
        day3,
        day4,
        day5,
        day6,
        day7,
        day8,
        day9,
        day10,
        day11,
        day12,
        day13,
        day14,
        day15,
        day16,
        day17,
        day18,
        day19,
        day20,
        day21,
        day22,
        day23,
        day24,
        day25,
    );