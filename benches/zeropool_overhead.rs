#![feature(test)]
extern crate test;

mod common;
use common::TASK_COUNT;

use std::hint::black_box;
use test::Bencher;
use zero_pool::ZeroPool;

struct IndexTask {
    result: *mut u64,
}

fn index_task_fn(params: &IndexTask) {
    unsafe {
        *params.result = 42u64;
    }
}

#[bench]
fn overhead(b: &mut Bencher) {
    let pool = ZeroPool::new();

    b.iter(|| {
        let mut results = vec![0u64; TASK_COUNT];

        let mut tasks = Vec::with_capacity(TASK_COUNT);
        for result in results.iter_mut().take(TASK_COUNT) {
            tasks.push(IndexTask { result });
        }

        pool.submit_batch_and_wait(index_task_fn, &tasks);

        black_box(results);
    });
}
