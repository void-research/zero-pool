#![feature(test)]
extern crate test;

mod common;
use common::{HEAVY_COMPUTE_WORK_AMOUNT, TASK_COUNT};

use rayon::prelude::*;
use std::hint::black_box;
use test::Bencher;

#[bench]
fn bench_heavy_compute_rayon(b: &mut Bencher) {
    let pool = rayon::ThreadPoolBuilder::new().build().unwrap();

    let seeds: Vec<u64> = (0..TASK_COUNT)
        .map(|i| {
            let mut seed = i as u64;
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            seed
        })
        .collect();

    b.iter(|| {
        let results: Vec<u64> = pool.install(|| {
            seeds
                .par_iter()
                .map(|&seed| {
                    let mut sum = 0u64;
                    let mut x = seed;

                    for _ in 0..HEAVY_COMPUTE_WORK_AMOUNT {
                        x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                        sum = sum.wrapping_add(x);

                        if x.is_multiple_of(3) {
                            sum = sum.wrapping_mul(17);
                        } else if x.is_multiple_of(7) {
                            sum = sum.wrapping_add(x >> 8);
                        }
                    }

                    sum
                })
                .collect()
        });

        black_box(results);
    });
}
