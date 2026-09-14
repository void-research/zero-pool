use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::thread::Thread;

use crate::{PaddedType, TaskFnPointer, TaskParamPointer};

pub struct TaskBatch {
    next_byte_offset: PaddedType<AtomicUsize>,
    pub next: PaddedType<AtomicPtr<TaskBatch>>,
    pub fn_ptr: TaskFnPointer,
    params_ptr: TaskParamPointer,
    param_stride: usize,
    params_total_bytes: usize,
    waiter: Option<(*const AtomicUsize, NonNull<Thread>)>,
    // used only by thread that takes ownership for reclamation
    // but because of retagging/aliasing rules needs either unsafecell or atomic to pass MIRI.
    // should be the same machine instruction regardless of choice with Relaxed ordering.
    pub retired_epoch: AtomicUsize,
    pub retired_next: AtomicPtr<TaskBatch>,
}

impl TaskBatch {
    #[inline]
    pub unsafe fn new<T>(
        task_fn: fn(&T),
        params: *const [T],
        waiter: Option<(*const AtomicUsize, NonNull<Thread>)>,
    ) -> *mut Self {
        Box::into_raw(Box::new(Self {
            next_byte_offset: PaddedType(AtomicUsize::new(0)),
            next: PaddedType(AtomicPtr::new(std::ptr::null_mut())),
            fn_ptr: unsafe { std::mem::transmute::<fn(&T), TaskFnPointer>(task_fn) },
            params_ptr: unsafe { NonNull::new_unchecked(params.cast::<T>().cast_mut()).cast() },
            param_stride: std::mem::size_of::<T>(),
            params_total_bytes: std::mem::size_of::<T>() * params.len(),
            waiter,
            retired_epoch: AtomicUsize::new(0),
            retired_next: AtomicPtr::new(std::ptr::null_mut()),
        }))
    }

    pub fn complete_many(&self, count: usize) {
        if let Some((counter, thread)) = self.waiter
            && unsafe { (*counter).fetch_sub(count, Ordering::Release) } == count
        {
            unsafe { thread.as_ref().unpark() };
        }
    }

    pub fn claim_next_param(&self) -> Option<TaskParamPointer> {
        let byte_offset = self
            .next_byte_offset
            .fetch_add(self.param_stride, Ordering::Relaxed);

        if byte_offset >= self.params_total_bytes {
            return None;
        }

        unsafe { Some(self.params_ptr.add(byte_offset)) }
    }
}
