#![feature(test)]
extern crate test;

mod common;
use common::INDIVIDUAL_TASK_COUNT;

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
fn individual_tasks(b: &mut Bencher) {
    let pool = ZeroPool::new();

    b.iter(|| {
        let mut results = vec![0u64; INDIVIDUAL_TASK_COUNT];
        let mut tasks = Vec::with_capacity(INDIVIDUAL_TASK_COUNT);
        let mut futures = Vec::with_capacity(INDIVIDUAL_TASK_COUNT);

        for result in &mut results {
            tasks.push(IndexTask { result });
        }

        for task in &tasks {
            let future = pool.submit_task(index_task_fn, task);
            futures.push(future);
        }

        for future in futures {
            future.wait();
        }

        black_box(results);
    });
}
