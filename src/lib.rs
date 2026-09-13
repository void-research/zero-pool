//! # Zero-Pool
//!
//! High-performance thread pool with:
//! - Safe scoped task execution without per-task heap allocation
//! - Lock-free MPMC queue with epoch-based memory reclamation
//! - Function pointer dispatch (no trait objects / virtual dispatch)
//! - Zero external dependencies
//!
//! ```rust
//! use zero_pool::ZeroPool;
//!
//! struct Params { value: u64, result: *mut u64 }
//!
//! fn task(p: &Params) { unsafe { *p.result = p.value * 2; } }
//!
//! let pool = ZeroPool::new();
//! let mut result = 0;
//! pool.run(task, &[Params { value: 42, result: &raw mut result }]);
//! assert_eq!(result, 84);
//! ```

mod pool;
mod queue;
mod retired_list;
mod scope;
mod task_batch;
mod worker;

use std::ptr::NonNull;
use std::sync::OnceLock;

pub use pool::ZeroPool;
pub use scope::Scope;

static GLOBAL_ZP: OnceLock<ZeroPool> = OnceLock::new();

/// Returns a reference to the lazily initialized global pool.
#[inline]
pub fn global_pool() -> &'static ZeroPool {
    GLOBAL_ZP.get_or_init(ZeroPool::new)
}

pub(crate) type TaskParamPointer = NonNull<u8>;
pub(crate) type TaskFnPointer = fn(TaskParamPointer);

#[repr(align(64))]
pub(crate) struct PaddedType<T>(pub T);

impl<T> std::ops::Deref for PaddedType<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
