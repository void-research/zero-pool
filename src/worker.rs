use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::{queue::Queue, retired_list::RetiredList};

pub fn spawn_worker(id: usize, queue: Arc<Queue>) -> JoinHandle<()> {
    thread::Builder::new()
        .name(format!("zp{id}"))
        .spawn(move || {
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

                    if let Some(completion) = &batch.completion {
                        let counter_done = unsafe {
                            (*completion.counter).fetch_sub(completed, std::sync::atomic::Ordering::AcqRel) == completed
                        };
                        if counter_done {
                            completion.thread.unpark();
                        }
                    }

                    retired.try_clean(&queue);
                }
            }
        })
        .expect("spawn failed")
}
