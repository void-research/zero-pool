use std::{
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
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
    let mut result = 0;

    pool.run(
        compute_task,
        &[TaskParams {
            value: 42,
            result: &raw mut result,
        }],
    );

    assert_eq!(result, 85);
}

// Excluded under Miri: `global_pool()` is stored in a static `OnceLock`, so its worker
// threads are intentionally not joined until process exit. Skipping this under Miri
// allows running the entire test suite with memory leak detection.
#[test]
#[cfg(not(miri))]
fn test_global_pool_usage() {
    let pool = zero_pool::global_pool();
    let mut result = 0;

    pool.run(
        compute_task,
        &[TaskParams {
            value: 21,
            result: &raw mut result,
        }],
    );

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
    let mut results = vec![0; count];

    let tasks: Vec<_> = results
        .iter_mut()
        .enumerate()
        .map(|(i, result)| TaskParams {
            value: i as u64,
            result,
        })
        .collect();

    pool.run(compute_task, &tasks);

    for (i, &res) in results.iter().enumerate() {
        assert_eq!(res, (i as u64) * 2 + 1, "Task {i} computed incorrect value");
    }
}

#[test]
fn test_empty_batch_submission() {
    let pool = ZeroPool::new();
    pool.run(compute_task, &[]);
}

#[test]
fn test_worker_counts() {
    for worker_count in [1, 2, 4] {
        let pool = ZeroPool::with_workers(NonZeroUsize::new(worker_count).unwrap());
        let task_count = worker_count * 10;
        let mut results = vec![0; task_count];

        let tasks: Vec<_> = results
            .iter_mut()
            .enumerate()
            .map(|(i, result)| TaskParams {
                value: i as u64,
                result,
            })
            .collect();

        pool.run(compute_task, &tasks);

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
        let mut results = [0; 20];

        let tasks: Vec<_> = results
            .iter_mut()
            .enumerate()
            .map(|(i, result)| TaskParams {
                value: i as u64,
                result,
            })
            .collect();

        pool.run(compute_task, &tasks);

        for (i, &res) in results.iter().enumerate() {
            assert_eq!(res, (i as u64) * 2 + 1);
        }
    }
}

#[test]
fn test_scope() {
    let pool = ZeroPool::new();
    let mut r1 = 0;
    let mut r2 = 0;

    let p1 = [TaskParams {
        value: 10,
        result: &raw mut r1,
    }];
    let p2 = [TaskParams {
        value: 20,
        result: &raw mut r2,
    }];

    pool.scope(|s| {
        s.run(compute_task, &p1);
        s.run(compute_task, &p2);
    });

    assert_eq!(r1, 21);
    assert_eq!(r2, 41);
}

#[test]
fn test_scope_wait_and_is_complete() {
    let pool = ZeroPool::new();
    let mut r1 = 0;
    let mut r2 = 0;

    // Phase 1
    let p1 = [TaskParams {
        value: 5,
        result: &raw mut r1,
    }];
    pool.scope(|s| {
        s.run(compute_task, &p1);
        s.wait();
        assert!(s.is_complete());
    });

    // Phase 2 using phase 1's output
    let p2 = [TaskParams {
        value: r1,
        result: &raw mut r2,
    }];
    pool.run(compute_task, &p2);

    assert_eq!(r1, 11);
    assert_eq!(r2, 23);
}

#[test]
fn test_scope_panic_safety() {
    let pool = ZeroPool::new();
    let mut r1 = 0;

    let p1 = [TaskParams {
        value: 10,
        result: &raw mut r1,
    }];

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pool.scope(|s| {
            s.run(compute_task, &p1);
            panic!("intentional panic inside scope");
        });
    }));

    assert!(result.is_err());
    assert_eq!(
        r1, 21,
        "Task should complete even when scope closure panics"
    );
}

#[test]
fn test_detached_submission() {
    let pool = ZeroPool::new();
    let completed = Arc::new(AtomicUsize::new(0));

    let ptr = Box::into_raw(Box::new(completed.clone()));
    unsafe {
        pool.run_detached(
            |counter: &Arc<AtomicUsize>| {
                counter.fetch_add(1, Ordering::Relaxed);
            },
            std::ptr::slice_from_raw_parts(ptr, 1),
        );
    }

    while completed.load(Ordering::Acquire) < 1 {
        std::thread::yield_now();
    }

    drop(pool);

    unsafe {
        drop(Box::from_raw(ptr));
    }
}

#[test]
fn test_detached_batch_submission() {
    let pool = ZeroPool::new();
    let completed = Arc::new(AtomicUsize::new(0));

    let count = 20;
    let batch: Box<[Arc<AtomicUsize>]> = (0..count).map(|_| completed.clone()).collect();
    let ptr = Box::into_raw(batch);

    unsafe {
        pool.run_detached(
            |counter: &Arc<AtomicUsize>| {
                counter.fetch_add(1, Ordering::Relaxed);
            },
            ptr,
        );
    }

    while completed.load(Ordering::Acquire) < count {
        std::thread::yield_now();
    }

    drop(pool);

    unsafe {
        drop(Box::from_raw(ptr));
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

        pool.run(compute_task, &tasks);

        for (i, &res) in results.iter().enumerate() {
            assert_eq!(res, ((round * 100 + i) as u64) * 2 + 1);
        }
    }
}

#[test]
fn test_reclaim_trigger() {
    // >256 submissions for each worker
    let pool = ZeroPool::with_workers(NonZeroUsize::new(2).unwrap());
    let mut result = 0;
    let params = [TaskParams {
        value: 1,
        result: &raw mut result,
    }];

    for _ in 0..550 {
        pool.run(compute_task, &params);
    }
    assert_eq!(result, 3);
}
