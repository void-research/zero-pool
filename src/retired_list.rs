use crate::queue::{EPOCH_MASK, EPOCH_MASK_HALF, Queue};
use crate::task_batch::TaskBatch;

pub struct RetiredList {
    head: *mut TaskBatch,
    tail: *mut TaskBatch,
    tick: u8,
}

impl RetiredList {
    pub fn new() -> Self {
        RetiredList {
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
            tick: 0,
        }
    }

    pub fn push(&mut self, batch: *mut TaskBatch, epoch: usize) {
        unsafe {
            (*batch).retired_epoch = epoch;

            if self.head.is_null() {
                self.head = batch;
            } else {
                (*self.tail).retired_next = batch;
            }
            self.tail = batch;
        }
    }

    pub fn clean(&mut self, queue: &Queue) {
        self.tick = self.tick.wrapping_add(1);
        if self.tick != 0 {
            return;
        }

        let safe_epoch = queue.advance_and_min_epoch();
        let mut current = self.head;

        // list is chronologically sorted; reclaim prefix only
        while !current.is_null() {
            let node_epoch = unsafe { (*current).retired_epoch };
            if safe_epoch.wrapping_sub(node_epoch).wrapping_sub(1) & EPOCH_MASK
                < (EPOCH_MASK_HALF - 1)
            {
                unsafe {
                    let next = (*current).retired_next;
                    drop(Box::from_raw(current));
                    current = next;
                }
            } else {
                break;
            }
        }

        self.head = current;
        if current.is_null() {
            self.tail = std::ptr::null_mut();
        }
    }
}

impl Drop for RetiredList {
    fn drop(&mut self) {
        let mut current = self.head;
        while !current.is_null() {
            unsafe {
                let next = (*current).retired_next;
                drop(Box::from_raw(current));
                current = next;
            }
        }
    }
}
