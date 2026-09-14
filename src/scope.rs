use crate::queue::Queue;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, Thread};

/// A scope for spawning concurrent tasks that borrow from the local stack.
///
/// Tasks are guaranteed to complete before [`ZeroPool::scope`](crate::ZeroPool::scope)
/// returns, even on panic.
pub struct Scope<'scope, 'env: 'scope> {
    queue: &'env Queue,
    counter: AtomicUsize,
    thread: Thread,
    _marker: PhantomData<(&'scope mut &'scope (), &'env mut &'env ())>,
}

impl<'scope, 'env> Scope<'scope, 'env> {
    pub(crate) fn new(queue: &'env Queue) -> Self {
        Self {
            queue,
            counter: AtomicUsize::new(0),
            thread: thread::current(),
            _marker: PhantomData,
        }
    }

    /// Submits tasks to the pool within this scope.
    #[inline]
    pub fn run<T: 'scope>(&self, task_fn: fn(&T), params: &'scope [T]) {
        if params.is_empty() {
            return;
        }
        self.counter.fetch_add(params.len(), Ordering::Relaxed);
        unsafe {
            self.queue.push_task_batch(
                task_fn,
                params,
                Some((&raw const self.counter, &raw const self.thread)),
            );
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
