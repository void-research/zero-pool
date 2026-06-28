use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::{queue::Queue, retired_list::RetiredList};

pub fn spawn_worker(id: usize, queue: Arc<Queue>) -> JoinHandle<()> {
    thread::Builder::new()
        .name(format!("zp{}", id))
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

                    batch.future.complete_many(completed);

                    retired.clean(&queue);
                }
            }
        })
        .expect("spawn failed")
}
