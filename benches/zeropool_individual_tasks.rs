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
        *params.result = 42;
    }
}

#[bench]
fn individual_tasks(b: &mut Bencher) {
    let pool = ZeroPool::new();

    b.iter(|| {
        let mut results = vec![0; INDIVIDUAL_TASK_COUNT];
        let mut tasks = Vec::with_capacity(INDIVIDUAL_TASK_COUNT);

        for result in &mut results {
            tasks.push(IndexTask { result });
        }

        pool.scope(|s| {
            for task in &tasks {
                s.run(index_task_fn, std::slice::from_ref(task));
            }
        });

        black_box(results);
    });
}
