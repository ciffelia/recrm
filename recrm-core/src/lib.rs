mod file;
mod job;
mod worker;

use crate::job::JobProgressEvent;
use crate::worker::WorkerPool;
use std::path::PathBuf;
use std::time::Duration;

pub fn recursively_remove(path: PathBuf) {
    let worker_pool = WorkerPool::new(num_cpus::get());
    worker_pool.queue(path);

    loop {
        let timeout = Duration::from_millis(500);
        let res = worker_pool
            .job_progress
            .event_receiver
            .recv_timeout(timeout);

        worker_pool.job_progress.print_progress();

        match res {
            Ok(JobProgressEvent::DeleteComplete) => {
                break;
            }
            Err(_err) => {
                continue;
            }
        }
    }

    worker_pool.terminate_workers().unwrap();
}
