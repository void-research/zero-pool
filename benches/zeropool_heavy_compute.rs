#![feature(test)]
extern crate test;

mod common;
use common::{HEAVY_COMPUTE_WORK_AMOUNT, TASK_COUNT};

use std::hint::black_box;
use test::Bencher;
use zero_pool::ZeroPool;

struct HeavyComputeTask {
    seed: u64,
    result: *mut u64,
}

fn heavy_compute_task_fn(params: &HeavyComputeTask) {
    let mut sum = 0u64;
    let mut x = params.seed;

    for _ in 0..HEAVY_COMPUTE_WORK_AMOUNT {
        x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        sum = sum.wrapping_add(x);

        if x.is_multiple_of(3) {
            sum = sum.wrapping_mul(17);
        } else if x.is_multiple_of(7) {
            sum = sum.wrapping_add(x >> 8);
        }
    }

    unsafe {
        *params.result = sum;
    }
}

#[bench]
fn heavy_compute(b: &mut Bencher) {
    let pool = ZeroPool::new();

    let seeds: Vec<u64> = (0..TASK_COUNT)
        .map(|i| {
            let mut seed = i as u64;
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            seed
        })
        .collect();

    b.iter(|| {
        let mut results = vec![0u64; TASK_COUNT];

        let mut tasks = Vec::with_capacity(TASK_COUNT);
        for (i, res) in results.iter_mut().enumerate() {
            tasks.push(HeavyComputeTask {
                seed: seeds[i],
                result: res,
            });
        }

        pool.submit_batch_and_wait(heavy_compute_task_fn, &tasks);

        black_box(results);
    });
}
