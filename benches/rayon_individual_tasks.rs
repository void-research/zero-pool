#![feature(test)]
extern crate test;

mod common;
use common::INDIVIDUAL_TASK_COUNT;

use std::hint::black_box;
use test::Bencher;

#[bench]
fn individual_tasks(b: &mut Bencher) {
    let pool = rayon::ThreadPoolBuilder::new().build().unwrap();

    b.iter(|| {
        let mut results: Vec<u64> = vec![0; INDIVIDUAL_TASK_COUNT];

        pool.install(|| {
            rayon::scope(|s| {
                for result in &mut results {
                    s.spawn(move |_| {
                        *result = 42;
                    });
                }
            });
        });

        black_box(results);
    });
}
