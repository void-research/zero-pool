use crate::{
    TaskFnPointer,
    queue::Queue,
    scope::{Scope, ScopeGuard},
    worker::spawn_worker,
};
use std::{
    num::NonZeroUsize,
    ptr::NonNull,
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

    /// Creates a scope for executing concurrent tasks on this pool.
    ///
    /// Tasks submitted via [`Scope::submit`] or [`Scope::submit_batch`] can safely borrow
    /// from the caller's stack frame, and are guaranteed to complete before `scope` returns.
    ///
    /// If the closure panics, the scope waits for all currently running tasks to finish
    /// before unwinding to preserve memory safety.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zero_pool::ZeroPool;
    ///
    /// struct TaskParams { value: u64, result: *mut u64 }
    /// fn compute(params: &TaskParams) {
    ///     unsafe { *params.result = params.value * 2; }
    /// }
    ///
    /// let pool = ZeroPool::new();
    /// let mut r1 = 0;
    /// let mut r2 = 0;
    /// let p1 = TaskParams { value: 10, result: &raw mut r1 };
    /// let p2 = TaskParams { value: 20, result: &raw mut r2 };
    ///
    /// pool.scope(|s| {
    ///     s.submit(compute, &p1);
    ///     s.submit(compute, &p2);
    /// });
    ///
    /// assert_eq!(r1, 20);
    /// assert_eq!(r2, 40);
    /// ```
    pub fn scope<'env, F, R>(&'env self, f: F) -> R
    where
        F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> R,
    {
        let scope = Scope::new(&self.queue);
        let guard = ScopeGuard::new(&scope);
        let result = f(&scope);
        drop(guard);
        result
    }

    /// Submits a single typed task and waits for it to complete.
    ///
    /// This is safe because `param` is guaranteed to outlive task execution.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zero_pool::ZeroPool;
    ///
    /// struct TaskParams { value: u64, result: *mut u64 }
    /// fn compute(params: &TaskParams) {
    ///     unsafe { *params.result = params.value * 2; }
    /// }
    ///
    /// let pool = ZeroPool::new();
    /// let mut result = 0;
    /// let params = TaskParams { value: 42, result: &raw mut result };
    ///
    /// pool.submit_and_wait(compute, &params);
    /// assert_eq!(result, 84);
    /// ```
    #[inline]
    pub fn submit_and_wait<T>(&self, task_fn: fn(&T), param: &T) {
        self.scope(|s| s.submit(task_fn, param));
    }

    /// Submits a batch of uniform tasks and waits for all of them to complete.
    ///
    /// All tasks in the batch are executed concurrently across worker threads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zero_pool::ZeroPool;
    ///
    /// struct TaskParams { value: u64, result: *mut u64 }
    /// fn compute(params: &TaskParams) {
    ///     unsafe { *params.result = params.value * 2; }
    /// }
    ///
    /// let pool = ZeroPool::new();
    /// let mut results = vec![0u64; 100];
    /// let tasks: Vec<_> = results
    ///     .iter_mut()
    ///     .enumerate()
    ///     .map(|(i, res)| TaskParams { value: i as u64, result: res })
    ///     .collect();
    ///
    /// pool.submit_batch_and_wait(compute, &tasks);
    /// assert_eq!(results[0], 0);
    /// assert_eq!(results[99], 198);
    /// ```
    #[inline]
    pub fn submit_batch_and_wait<T>(&self, task_fn: fn(&T), params: &[T]) {
        self.scope(|s| s.submit_batch(task_fn, params));
    }

    /// Submits a single task without tracking or waiting for its completion.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory pointed to by `param` remains valid
    /// until the worker thread has finished executing the task function.
    pub unsafe fn submit_detached<T>(&self, task_fn: fn(&T), param: *const T) {
        unsafe {
            self.submit_detached_batch(task_fn, param, 1);
        }
    }

    /// Submits a batch of uniform tasks without tracking or waiting for their completion.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory pointed to by `params` remains valid
    /// until all `count` tasks have finished executing.
    pub unsafe fn submit_detached_batch<T>(&self, task_fn: fn(&T), params: *const T, count: usize) {
        if count == 0 || params.is_null() {
            return;
        }

        unsafe {
            self.queue.push_task_batch(
                std::mem::transmute::<fn(&T), TaskFnPointer>(task_fn),
                NonNull::new_unchecked(params as *mut T).cast(),
                std::mem::size_of::<T>(),
                std::mem::size_of::<T>() * count,
                count,
                None,
            );
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
