use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::{queue::Queue, retired_list::RetiredList, task_future::TaskFuture};

pub fn spawn_worker(id: usize, queue: Arc<Queue>, latch: TaskFuture) -> JoinHandle<()> {
    thread::Builder::new()
        .name(format!("zp{}", id))
        .spawn(move || {
            // register this thread with the queue's waiter so it can be unparked by id
            queue.register_worker_thread(id);
            // signal registration complete and wait for all workers + main
            latch.complete_many(1);
            drop(latch);

            let mut retired = RetiredList::new();

            loop {
                if !queue.wait_for_work(id) {
                    break;
                }

                while let Some((batch, first_param)) = queue.get_next_batch(id, &mut retired) {
                    let mut completed = 1;
                    (batch.fn_ptr)(first_param);

                    while let Some(param) = batch.claim_next_param() {
                        (batch.fn_ptr)(param);
                        completed += 1;
                    }

                    batch.future.complete_many(completed);

                    retired.clean(&queue);
                }
            }
        })
        .expect("spawn failed")
}
