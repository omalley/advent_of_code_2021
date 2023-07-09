use criterion::{criterion_group, criterion_main, Criterion};
use omalley_aoc2022::{INPUTS, NAMES};

macro_rules! benchmarks {
    ($day:ident) => {

        fn benchmark_function(c: &mut Criterion) {
            use omalley_aoc2022::$day;
            let posn = NAMES.iter().position(|n| *n == stringify!($day)).expect("Unknown day");
            let input = $day::generator(INPUTS[posn]);
            c.bench_function(concat!(stringify!($day), " gen"), |b| {
                b.iter(|| $day::generator(INPUTS[posn]))
            });
            c.bench_function(concat!(stringify!($day), " part 1"), |b| {
                b.iter(|| $day::part1(&input))
            });
            c.bench_function(concat!(stringify!($day), " part 2"), |b| {
                b.iter(|| $day::part2(&input))
            });
        }
    };
}

benchmarks!(day19);

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
