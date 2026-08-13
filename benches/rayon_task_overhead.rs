#![feature(test)]
extern crate test;

mod common;
use common::TASK_COUNT;

use rayon::prelude::*;
use std::hint::black_box;
use test::Bencher;

#[bench]
fn bench_task_overhead_rayon(b: &mut Bencher) {
    let pool = rayon::ThreadPoolBuilder::new().build().unwrap();

    b.iter(|| {
        let results: Vec<u64> =
            pool.install(|| (0..TASK_COUNT).into_par_iter().map(|_| 42u64).collect());

        black_box(results);
    });
}
