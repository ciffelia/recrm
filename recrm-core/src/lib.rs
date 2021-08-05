mod event;
mod file;
mod job;
mod worker;

pub use crate::event::Event;
use crate::file::{File, NewFileOptions};
use crate::job::{JobProgress, JobProgressStore, JobQueue};
use crate::worker::{StartWorkerOption, WorkerPool};
use anyhow::{bail, Result};
use log::info;
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationMode {
    Idle,
    Scan,
    ScanAndDelete,
}

pub struct Task {
    mode: OperationMode,
    worker_pool: WorkerPool,
    job_queue: JobQueue,
    job_progress_store: JobProgressStore,
    num_cpus: usize,
}

impl Task {
    pub fn new<T: AsRef<Path>>(file_path: T) -> Result<Task> {
        let file_path = file_path.as_ref();
        if !file_path.is_dir() {
            bail!("{} is not directory.", file_path.display())
        }

        let file = File::new(NewFileOptions {
            path: file_path,
            is_dir: true,
            parent: None,
        });

        let task = Task {
            mode: OperationMode::Idle,
            worker_pool: WorkerPool::new(),
            job_queue: JobQueue::new(),
            job_progress_store: JobProgressStore::new(),
            num_cpus: num_cpus::get(),
        };

        task.job_queue
            .scan_sender()
            .send(Arc::new(Mutex::new(file)))
            .unwrap();
        task.job_progress_store.increment_dir_found();

        Ok(task)
    }

    pub fn set_mode(&mut self, mode: OperationMode) -> Result<()> {
        if mode == self.mode {
            return Ok(());
        }

        self.worker_pool.terminate_workers()?;

        match mode {
            OperationMode::Idle => {}
            OperationMode::Scan => {
                self.start_workers(false);
            }
            OperationMode::ScanAndDelete => {
                self.start_workers(true);
            }
        }

        self.mode = mode;

        info!("set mode: {:?}", mode);

        Ok(())
    }

    pub fn wait_for_event(&self, timeout: Duration) -> Option<Event> {
        self.worker_pool.wait_for_event(timeout)
    }

    pub fn progress(&self) -> JobProgress {
        self.job_progress_store.progress()
    }

    fn start_workers(&mut self, allow_delete: bool) {
        for _ in 0..self.num_cpus {
            self.worker_pool.start_worker(StartWorkerOption {
                allow_delete,
                job_queue: self.job_queue.clone(),
                job_progress_store: self.job_progress_store.clone(),
            });
        }
    }
}
