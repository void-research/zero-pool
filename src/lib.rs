//! # Zero-Pool: Ultra-High Performance Thread Pool
//!
//! A thread pool implementation designed for maximum performance through:
//! - Safe scoped task execution without per-task allocation
//! - Zero-result-transport pattern (direct writes to caller memory)
//! - Single global queue with optimal load balancing
//! - Function pointer dispatch (no trait objects)
//! - Lock-free queue operations with event-based worker coordination
//!
//! ## Example
//!
//! ```rust
//! use zero_pool::ZeroPool;
//!
//! struct MyTaskParams { value: u64, result: *mut u64 }
//!
//! fn my_task(params: &MyTaskParams) {
//!     unsafe { *params.result = params.value * 2; }
//! }
//!
//! let pool = ZeroPool::new();
//! let mut result = 0u64;
//! let task_params = MyTaskParams { value: 42, result: &raw mut result };
//! pool.submit_and_wait(my_task, &task_params);
//! assert_eq!(result, 84);
//! ```

mod padded_type;
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
///
/// This is the simplest way to share a single pool across your application.
/// The pool is created on first use using [`ZeroPool::new`].
#[inline]
pub fn global_pool() -> &'static ZeroPool {
    GLOBAL_ZP.get_or_init(ZeroPool::new)
}

/// Non-null pointer to task parameter struct
///
/// This is type-erased for uniform storage but must be cast back
/// to the original parameter type within the task function.
pub(crate) type TaskParamPointer = NonNull<u8>;

/// Function pointer type for task execution
///
/// Tasks receive a non-null pointer to their parameter struct and must
/// cast it to the appropriate type for safe access.
pub(crate) type TaskFnPointer = fn(TaskParamPointer);
