mod file;
mod job;
mod worker;

use crate::job::JobProgressEvent;
use crate::worker::WorkerPool;
use log::info;
use std::env;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn main() {
    env_logger::init();

    {
        let start = Instant::now();

        let input = env::args().nth(1).expect("No file specified");

        let worker_pool = WorkerPool::new(num_cpus::get());
        worker_pool.queue(PathBuf::from(input));

        loop {
            let timeout = Duration::from_millis(500);
            let res = worker_pool
                .job_progress
                .event_receiver
                .recv_timeout(timeout);

            match res {
                Ok(JobProgressEvent::DeleteComplete) => {
                    worker_pool.job_progress.print_progress();
                    break;
                }
                Err(_err) => {
                    worker_pool.job_progress.print_progress();
                    continue;
                }
            }
        }

        worker_pool.terminate_workers().unwrap();

        let end = start.elapsed();
        info!(
            "elapsed: {}.{:03}",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );
    }
}
