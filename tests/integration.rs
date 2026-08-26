use std::num::NonZeroUsize;
use zero_pool::ZeroPool;

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
    pool.submit_and_wait(compute_task, &params);

    assert_eq!(result, 85);
}

// Excluded under Miri: `global_pool()` is stored in a static `OnceLock`, so its worker
// threads are intentionally not joined until process exit. Skipping this under Miri
// allows running the entire test suite with memory leak detection.
#[test]
#[cfg(not(miri))]
fn test_global_pool_usage() {
    let pool = zero_pool::global_pool();
    let mut result = 0u64;

    let params = TaskParams {
        value: 21,
        result: &raw mut result,
    };

    pool.submit_and_wait(compute_task, &params);

    assert_eq!(result, 43);
    assert!(
        std::ptr::eq(pool, zero_pool::global_pool()),
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

    pool.submit_batch_and_wait(compute_task, &tasks);

    for (i, &res) in results.iter().enumerate() {
        assert_eq!(res, (i as u64) * 2 + 1, "Task {i} computed incorrect value");
    }
}

#[test]
fn test_empty_batch_submission() {
    let pool = ZeroPool::new();
    let empty_tasks: Vec<TaskParams> = Vec::new();
    pool.submit_batch_and_wait(compute_task, &empty_tasks);
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

        pool.submit_batch_and_wait(compute_task, &tasks);

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

        pool.submit_batch_and_wait(compute_task, &tasks);

        for (i, &res) in results.iter().enumerate() {
            assert_eq!(res, (i as u64) * 2 + 1);
        }
    }
}

#[test]
fn test_scope() {
    let pool = ZeroPool::new();
    let mut r1 = 0u64;
    let mut r2 = 0u64;
    let p1 = TaskParams {
        value: 10,
        result: &raw mut r1,
    };
    let p2 = TaskParams {
        value: 20,
        result: &raw mut r2,
    };

    pool.scope(|s| {
        s.submit(compute_task, &p1);
        s.submit(compute_task, &p2);
    });

    assert_eq!(r1, 21);
    assert_eq!(r2, 41);
}

#[test]
fn test_detached_submission() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let pool = ZeroPool::new();
    let done = Arc::new(AtomicBool::new(false));

    struct DetachedTask {
        done: Arc<AtomicBool>,
    }

    fn detached_fn(params: &DetachedTask) {
        params.done.store(true, Ordering::Release);
    }

    let task = Box::new(DetachedTask { done: done.clone() });
    let task_ptr = Box::into_raw(task);

    unsafe {
        pool.submit_detached(detached_fn, task_ptr);
    }

    while !done.load(Ordering::Acquire) {
        std::thread::yield_now();
    }

    unsafe {
        drop(Box::from_raw(task_ptr));
    }
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

        pool.submit_batch_and_wait(compute_task, &tasks);

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
        pool.submit_and_wait(compute_task, &params);
    }
    assert_eq!(result, 3);
}
