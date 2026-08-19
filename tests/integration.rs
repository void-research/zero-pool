use std::{num::NonZeroUsize, time::Duration};
use zero_pool::{ZeroPool, global_pool};

struct TaskParams {
    value: u64,
    result: *mut u64,
}

fn compute_task(params: &TaskParams) {
    unsafe {
        *params.result = params.value * 2 + 1;
    }
}

#[test]
fn test_basic_functionality() {
    let pool = ZeroPool::new();
    let mut result = 0u64;

    let params = TaskParams {
        value: 42,
        result: &raw mut result,
    };
    let future = pool.submit_task(compute_task, &params);
    future.wait();

    assert!(future.is_complete());
    assert_eq!(result, 85);
}

#[test]
fn test_global_pool_usage() {
    let pool = global_pool();
    let mut result = 0u64;

    let params = TaskParams {
        value: 21,
        result: &raw mut result,
    };

    let future = pool.submit_task(compute_task, &params);
    future.wait();

    assert_eq!(result, 43);
    assert!(
        std::ptr::eq(pool, global_pool()),
        "Global pool should be a singleton"
    );
}

#[test]
fn test_batch_submission() {
    let pool = ZeroPool::new();
    let count = 100;
    let mut results = vec![0u64; count];

    let tasks: Vec<_> = results
        .iter_mut()
        .enumerate()
        .map(|(i, result)| TaskParams {
            value: i as u64,
            result,
        })
        .collect();

    let batch = pool.submit_batch(compute_task, &tasks);
    batch.wait();

    assert!(batch.is_complete());
    for (i, &res) in results.iter().enumerate() {
        assert_eq!(res, (i as u64) * 2 + 1, "Task {i} computed incorrect value");
    }
}

#[test]
fn test_empty_batch_submission() {
    let pool = ZeroPool::new();
    let empty_tasks: Vec<TaskParams> = Vec::new();
    let empty_batch = pool.submit_batch(compute_task, &empty_tasks);

    assert!(empty_batch.is_complete());
    empty_batch.wait();
}

#[test]
fn test_worker_counts() {
    for worker_count in [1, 2, 4] {
        let pool = ZeroPool::with_workers(NonZeroUsize::new(worker_count).unwrap());
        let task_count = worker_count * 10;
        let mut results = vec![0u64; task_count];

        let tasks: Vec<_> = results
            .iter_mut()
            .enumerate()
            .map(|(i, result)| TaskParams {
                value: i as u64,
                result,
            })
            .collect();

        let batch = pool.submit_batch(compute_task, &tasks);
        batch.wait();

        for (i, &res) in results.iter().enumerate() {
            assert_eq!(
                res,
                (i as u64) * 2 + 1,
                "Worker count {worker_count}, task {i} failed"
            );
        }
    }
}

#[test]
fn test_pool_lifecycle_and_recreation() {
    for _ in 0..3 {
        let pool = ZeroPool::new();
        let mut results = [0u64; 20];

        let tasks: Vec<_> = results
            .iter_mut()
            .enumerate()
            .map(|(i, result)| TaskParams {
                value: i as u64,
                result,
            })
            .collect();

        let batch = pool.submit_batch(compute_task, &tasks);
        batch.wait();

        for (i, &res) in results.iter().enumerate() {
            assert_eq!(res, (i as u64) * 2 + 1);
        }
    }
}

#[test]
fn test_wait_timeout() {
    let pool = ZeroPool::new();
    let mut result = 0u64;

    let params = TaskParams {
        value: 10,
        result: &raw mut result,
    };
    let future = pool.submit_task(compute_task, &params);

    let completed = future.wait_timeout(Duration::from_secs(5));
    assert!(completed, "Task should complete within timeout");
    assert_eq!(result, 21);

    // Empty batch should return true immediately on wait_timeout
    let empty_tasks: Vec<TaskParams> = Vec::new();
    let empty_batch = pool.submit_batch(compute_task, &empty_tasks);
    assert!(empty_batch.wait_timeout(Duration::from_millis(50)));
}

#[test]
fn test_consecutive_batches() {
    let pool = ZeroPool::new();
    let batch_size = 10;

    for round in 0..3 {
        let mut results = vec![0u64; batch_size];
        let tasks: Vec<_> = results
            .iter_mut()
            .enumerate()
            .map(|(i, result)| TaskParams {
                value: (round * 100 + i) as u64,
                result,
            })
            .collect();

        let batch = pool.submit_batch(compute_task, &tasks);
        batch.wait();

        for (i, &res) in results.iter().enumerate() {
            assert_eq!(res, ((round * 100 + i) as u64) * 2 + 1);
        }
    }
}

#[test]
fn test_reclaim_trigger() {
    // >256 submissions for each worker
    let pool = ZeroPool::with_workers(NonZeroUsize::new(2).unwrap());
    let mut result = 0u64;
    let params = TaskParams {
        value: 1,
        result: &raw mut result,
    };

    for _ in 0..550 {
        let future = pool.submit_task(compute_task, &params);
        future.wait();
    }
    assert_eq!(result, 3);
}
