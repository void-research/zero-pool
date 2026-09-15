use crate::queue::Queue;
use crate::task_batch::TaskBatch;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, Thread};

/// A scope for spawning concurrent tasks that borrow from the local stack.
///
/// All submitted tasks are guaranteed to complete before the scope exits.
pub struct Scope<'scope, 'env: 'scope> {
    counter: AtomicUsize,
    queue: &'env Queue,
    thread: Thread,
    _marker: PhantomData<(&'scope mut &'scope (), &'env mut &'env ())>,
}

impl<'scope, 'env> Scope<'scope, 'env> {
    pub(crate) fn new(queue: &'env Queue) -> Self {
        Self {
            counter: AtomicUsize::new(0),
            queue,
            thread: thread::current(),
            _marker: PhantomData,
        }
    }

    /// Submits tasks to the pool within this scope.
    #[inline]
    pub fn run<T: 'scope>(&self, task_fn: fn(&T), params: &'scope [T]) {
        if !params.is_empty() {
            self.counter.fetch_add(params.len(), Ordering::Relaxed);
            let batch = unsafe {
                TaskBatch::new(
                    task_fn,
                    params,
                    Some((&raw const self.counter, self.thread.clone())),
                )
            };
            self.queue.enqueue(batch, params.len());
        }
    }

    /// Waits for all tasks currently submitted to this scope to complete.
    ///
    /// Can be called multiple times to synchronize between phases of work.
    #[inline]
    pub fn wait(&self) {
        while self.counter.load(Ordering::Acquire) != 0 {
            thread::park();
        }
    }

    /// Checks if all submitted tasks have finished.
    #[must_use]
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.counter.load(Ordering::Acquire) == 0
    }
}

/// Guard ensuring all scoped tasks complete even if unwinding due to a panic.
pub(crate) struct ScopeGuard<'s, 'scope, 'env>(pub &'s Scope<'scope, 'env>);

impl Drop for ScopeGuard<'_, '_, '_> {
    fn drop(&mut self) {
        self.0.wait();
    }
}
