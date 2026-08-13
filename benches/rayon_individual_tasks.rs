#![feature(test)]
extern crate test;

mod common;
use common::INDIVIDUAL_TASK_COUNT;

use std::hint::black_box;
use test::Bencher;

#[bench]
fn bench_individual_tasks_rayon(b: &mut Bencher) {
    let pool = rayon::ThreadPoolBuilder::new().build().unwrap();

    b.iter(|| {
        let mut results = vec![0u64; INDIVIDUAL_TASK_COUNT];

        pool.install(|| {
            rayon::scope(|s| {
                for result in &mut results {
                    s.spawn(move |_| {
                        *result = 42u64;
                    });
                }
            });
        });

        black_box(results);
    });
}
