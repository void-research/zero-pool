use std::{
    sync::{Arc, atomic::Ordering},
    thread::{self, JoinHandle},
};

use crate::{
    queue::{EPOCH_MASK, EPOCH_MASK_HALF, Queue},
    task_batch::TaskBatch,
    task_future::TaskFuture,
};

pub fn spawn_worker(id: usize, queue: Arc<Queue>, latch: TaskFuture) -> JoinHandle<()> {
    thread::Builder::new()
        .name(format!("zp{}", id))
        .spawn(move || {
            // register this thread with the queue's waiter so it can be unparked by id
            queue.register_worker_thread(id);
            // signal registration complete and wait for all workers + main
            latch.complete_many(1);
            drop(latch);

            let mut retired_head: *mut TaskBatch = std::ptr::null_mut();
            let mut retired_tail: *mut TaskBatch = std::ptr::null_mut();
            let mut local_tick: u8 = 0;

            loop {
                if !queue.wait_for_work(id) {
                    break;
                }

                while let Some((batch, first_param)) =
                    queue.get_next_batch(id, &mut retired_head, &mut retired_tail)
                {
                    let mut completed = 1;
                    (batch.fn_ptr)(first_param);

                    while let Some(param) = batch.claim_next_param() {
                        (batch.fn_ptr)(param);
                        completed += 1;
                    }

                    batch.future.complete_many(completed);

                    maybe_clean_local_retired(
                        &queue,
                        &mut local_tick,
                        &mut retired_head,
                        &mut retired_tail,
                    );
                }
            }

            // worker thread exits
            drain_retired(&mut retired_head);
        })
        .expect("spawn failed")
}

fn drain_retired(retired_head: &mut *mut TaskBatch) {
    let mut current = *retired_head;
    while !current.is_null() {
        unsafe {
            let next = (*current).retired_next.load(Ordering::Relaxed);
            drop(Box::from_raw(current));
            current = next;
        }
    }
    *retired_head = std::ptr::null_mut();
}

fn maybe_clean_local_retired(
    queue: &Queue,
    local_tick: &mut u8,
    retired_head: &mut *mut TaskBatch,
    retired_tail: &mut *mut TaskBatch,
) {
    *local_tick = local_tick.wrapping_add(1);
    if *local_tick != 0 {
        return;
    }

    clean_local_retired(queue, retired_head, retired_tail);
}

fn clean_local_retired(
    queue: &Queue,
    retired_head: &mut *mut TaskBatch,
    retired_tail: &mut *mut TaskBatch,
) {
    let safe_epoch = queue.advance_and_min_epoch();
    let mut current = *retired_head;

    // list is chronologically sorted; reclaim prefix only
    while !current.is_null() {
        let node_epoch = unsafe { (*current).retired_epoch.load(Ordering::Relaxed) };
        if safe_epoch.wrapping_sub(node_epoch).wrapping_sub(1) & EPOCH_MASK < (EPOCH_MASK_HALF - 1)
        {
            unsafe {
                let next = (*current).retired_next.load(Ordering::Relaxed);
                drop(Box::from_raw(current));
                current = next;
            }
        } else {
            break;
        }
    }

    *retired_head = current;
    if current.is_null() {
        *retired_tail = std::ptr::null_mut();
    }
}
