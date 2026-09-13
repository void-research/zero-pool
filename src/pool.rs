use crate::{
    queue::Queue,
    scope::{Scope, ScopeGuard},
    worker::spawn_worker,
};
use std::{
    num::NonZeroUsize,
    sync::Arc,
    thread::{self, JoinHandle},
};

pub struct ZeroPool {
    queue: Arc<Queue>,
    workers: Box<[JoinHandle<()>]>,
}

impl ZeroPool {
    /// Creates a new thread pool with worker count equal to available parallelism
    ///
    /// Worker count is determined by `std::thread::available_parallelism()`,
    /// falling back to 1 if unavailable. This is usually the optimal choice
    /// for CPU-bound workloads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zero_pool::ZeroPool;
    /// let pool = ZeroPool::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let worker_count = thread::available_parallelism().unwrap_or(NonZeroUsize::MIN);
        Self::with_workers(worker_count)
    }

    /// Creates a new thread pool with the specified number of workers
    ///
    /// Use this when you need precise control over the worker count,
    /// for example when coordinating with other thread pools or
    /// when you know the optimal count for your specific workload.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::num::NonZeroUsize;
    /// use zero_pool::ZeroPool;
    /// let pool = ZeroPool::with_workers(NonZeroUsize::new(4).unwrap());
    /// ```
    #[must_use]
    pub fn with_workers(worker_count: NonZeroUsize) -> Self {
        let worker_count = worker_count.get();
        let queue = Arc::new(Queue::new(worker_count));

        let workers = (0..worker_count)
            .map(|id| {
                let handle = spawn_worker(id, queue.clone());
                queue.register_thread(id, handle.thread().clone());
                handle
            })
            .collect();

        ZeroPool { queue, workers }
    }

    /// Creates a scope for executing concurrent tasks that can borrow from the stack.
    ///
    /// All tasks are guaranteed to complete before `scope` returns, even on panic.
    ///
    /// ```rust
    /// use zero_pool::ZeroPool;
    ///
    /// struct Params { value: u64, result: *mut u64 }
    /// fn compute(p: &Params) { unsafe { *p.result = p.value * 2; } }
    ///
    /// let pool = ZeroPool::new();
    /// let mut r1 = 0;
    /// let mut r2 = 0;
    ///
    /// let p1 = [Params { value: 10, result: &raw mut r1 }];
    /// let p2 = [Params { value: 20, result: &raw mut r2 }];
    ///
    /// pool.scope(|s| {
    ///     s.run(compute, &p1);
    ///     s.run(compute, &p2);
    /// });
    ///
    /// assert_eq!(r1, 20);
    /// assert_eq!(r2, 40);
    /// ```
    #[inline]
    pub fn scope<'env, F, R>(&'env self, f: F) -> R
    where
        F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> R,
    {
        let scope = Scope::new(&self.queue);
        let _guard = ScopeGuard(&scope);
        f(&scope)
    }

    /// Submits tasks and waits for all to complete.
    ///
    /// ```rust
    /// use zero_pool::ZeroPool;
    ///
    /// struct Params { value: u64, result: *mut u64 }
    /// fn compute(p: &Params) { unsafe { *p.result = p.value * 2; } }
    ///
    /// let pool = ZeroPool::new();
    /// let mut result = 0;
    /// pool.run(compute, &[Params { value: 42, result: &raw mut result }]);
    /// assert_eq!(result, 84);
    /// ```
    #[inline]
    pub fn run<T>(&self, task_fn: fn(&T), params: &[T]) {
        self.scope(|s| s.run(task_fn, params));
    }

    /// Submits tasks without waiting for completion.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `params` remains valid until all tasks finish.
    #[inline]
    pub unsafe fn run_detached<T>(&self, task_fn: fn(&T), params: *const [T]) {
        unsafe {
            self.queue
                .push_task_batch(task_fn, params, std::ptr::null(), None);
        }
    }
}

impl Default for ZeroPool {
    /// Creates a new thread pool with default settings
    ///
    /// Equivalent to calling `ZeroPool::new()`. Worker count is determined by
    /// `std::thread::available_parallelism()`, falling back to 1 if unavailable.
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ZeroPool {
    fn drop(&mut self) {
        self.queue.shutdown();

        for handle in std::mem::take(&mut self.workers) {
            let _ = handle.join();
        }
    }
}
