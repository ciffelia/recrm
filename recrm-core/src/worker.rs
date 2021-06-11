use crate::file::File;
use crate::job::{JobPool, JobType};
use crate::job::{JobProgress, JobProgressEvent};
use crossbeam::{channel, select};
use log::{debug, info, trace};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use std::{io, thread};

pub struct Worker {
    id: usize,
    preferred_job_type: JobType,
    job_pool: JobPool,
    job_progress: JobProgress,
    command_receiver: channel::Receiver<WorkerCommand>,
}

impl Worker {
    pub fn work_loop(&self) -> io::Result<()> {
        debug!("Worker {} started", self.id);

        loop {
            match self.command_receiver.try_recv() {
                Ok(WorkerCommand::Terminate) => {
                    break;
                }
                _ => {}
            }

            match self.preferred_job_type {
                JobType::Scan => {
                    if let Ok(file) = self.job_pool.scan_receiver.try_recv() {
                        self.scan(file)?;
                        continue;
                    }
                }
                JobType::Delete => {
                    if let Ok(file) = self.job_pool.delete_receiver.try_recv() {
                        self.delete(file)?;
                        continue;
                    }
                }
            };

            select! {
                recv(self.command_receiver) -> command => {
                    match command.unwrap() {
                        WorkerCommand::Terminate => {
                            break;
                        }
                    }
                }
                recv(self.job_pool.scan_receiver) -> file => {
                    self.scan(file.unwrap())?;
                }
                recv(self.job_pool.delete_receiver) -> file => {
                    self.delete(file.unwrap())?;
                }
            }
        }

        debug!("Worker {} exited", self.id);

        Ok(())
    }

    fn scan(&self, file: Arc<Mutex<File>>) -> io::Result<()> {
        trace!("[{:02}][receive_scan] {:?}", self.id, file.lock().path);

        let children = File::scan_children(&file)?;
        if children.len() == 0 {
            self.queue_delete(file);
        } else {
            for child in children {
                if child.is_dir {
                    self.queue_scan(Arc::new(Mutex::new(child)));
                    self.job_progress.report_dir_found();
                } else {
                    self.queue_delete(Arc::new(Mutex::new(child)));
                    self.job_progress.report_file_found();
                }
            }
        }

        Ok(())
    }

    fn delete(&self, file: Arc<Mutex<File>>) -> io::Result<()> {
        trace!("[{:02}][receive_delete] {:?}", self.id, file.lock().path);

        {
            let file = file.lock();
            file.delete()?;

            if file.is_dir {
                self.job_progress.report_dir_deleted();
            } else {
                self.job_progress.report_file_deleted();
            }

            match &file.parent {
                Some(parent) => {
                    if parent.lock().children_count == Some(0) {
                        self.queue_delete(parent.clone());
                    }
                }
                None => {
                    self.job_progress
                        .event_sender
                        .send(JobProgressEvent::DeleteComplete)
                        .unwrap();
                }
            }
        }

        Ok(())
    }

    fn queue_scan(&self, file: Arc<Mutex<File>>) {
        trace!("[{:02}][queue_scan] {:?}", self.id, file.lock().path);
        self.job_pool.scan_sender.send(file).unwrap();
    }

    fn queue_delete(&self, file: Arc<Mutex<File>>) {
        trace!("[{:02}][queue_delete] {:?}", self.id, file.lock().path);
        self.job_pool.delete_sender.send(file).unwrap();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WorkerCommand {
    Terminate,
}

pub struct WorkerPool {
    pub job_pool: JobPool,
    pub job_progress: JobProgress,
    command_senders: Vec<channel::Sender<WorkerCommand>>,
}

impl WorkerPool {
    pub fn new(size: usize) -> WorkerPool {
        let mut worker_pool = WorkerPool {
            job_pool: JobPool::new(),
            job_progress: JobProgress::new(),
            command_senders: vec![],
        };

        for id in 0..size {
            let preferred_job_type = if (id as f32) < (size as f32) / 2.0 {
                JobType::Scan
            } else {
                JobType::Delete
            };

            worker_pool.start_worker(id, preferred_job_type);
        }

        worker_pool
    }

    pub fn start_worker(&mut self, id: usize, preferred_job_type: JobType) {
        let job_pool = self.job_pool.clone();
        let job_progress = self.job_progress.clone();

        let (command_sender, command_receiver) = channel::unbounded();
        self.command_senders.push(command_sender);

        let worker = Worker {
            id,
            preferred_job_type,
            job_pool,
            job_progress,
            command_receiver,
        };

        thread::spawn(move || {
            worker.work_loop().unwrap();
        });
    }

    pub fn terminate_workers(&self) -> thread::Result<()> {
        debug!("Terminating workers");

        for sender in &self.command_senders {
            sender.send(WorkerCommand::Terminate).unwrap();
        }

        Ok(())
    }

    pub fn queue(&self, path: PathBuf) {
        info!("Queued {}", path.display());

        let file = File::new(path, None);
        let is_dir = file.is_dir;
        let msg = Arc::new(Mutex::new(file));

        if is_dir {
            self.job_pool.scan_sender.send(msg).unwrap();
            self.job_progress.report_dir_found();
        } else {
            self.job_pool.delete_sender.send(msg).unwrap();
            self.job_progress.report_file_found()
        }
    }
}
