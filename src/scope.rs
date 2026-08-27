use crate::TaskFnPointer;
use crate::queue::Queue;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, Thread};

/// A scope for spawning concurrent tasks that borrow from the local stack.
///
/// Tasks spawned via a `Scope` are guaranteed to complete before [`ZeroPool::scope`](crate::ZeroPool::scope)
/// returns, even in the event of a panic. This allows tasks to safely borrow data from the caller's stack frame.
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

    /// Submits a single typed task to the pool within this scope.
    #[inline]
    pub fn submit<T: 'scope>(&self, task_fn: fn(&T), param: &'scope T) {
        self.submit_batch(task_fn, std::slice::from_ref(param));
    }

    /// Submits a batch of uniform tasks to the pool within this scope.
    pub fn submit_batch<T: 'scope>(&self, task_fn: fn(&T), params: &'scope [T]) {
        if params.is_empty() {
            return;
        }

        self.counter.fetch_add(params.len(), Ordering::Relaxed);

        self.queue.push_task_batch(
            unsafe { std::mem::transmute::<fn(&T), TaskFnPointer>(task_fn) },
            NonNull::from(params).cast(),
            std::mem::size_of::<T>(),
            std::mem::size_of_val(params),
            params.len(),
            &self.counter,
            Some(self.thread.clone()),
        );
    }

    /// Waits for all tasks currently submitted to this scope to complete.
    ///
    /// This can be called multiple times within a scope to synchronize intermediate
    /// phases of work. Any remaining tasks will also be automatically waited for when
    /// the scope closes.
    #[inline]
    pub fn wait(&self) {
        while self.counter.load(Ordering::Acquire) != 0 {
            thread::park();
        }
    }

    /// Checks if all tasks currently submitted to this scope have finished.
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
