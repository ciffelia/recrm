use crate::event::Event;
use crate::file::File;
use crate::job::{DeleteJob, JobProgressStore, JobQueue, ScanJob};
use anyhow::Result;
use crossbeam::{channel, select};
use log::{debug, trace};
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct Worker {
    id: usize,
    allow_delete: bool,
    job_queue: JobQueue,
    job_progress_store: JobProgressStore,
    command_receiver: channel::Receiver<WorkerCommand>,
    event_sender: channel::Sender<Event>,
}

impl Worker {
    pub fn work_loop(&self) -> Result<()> {
        debug!("[{:02}] started", self.id);

        let never = channel::never();

        let scan_receiver = self.job_queue.scan_receiver();
        let delete_receiver = if self.allow_delete {
            self.job_queue.delete_receiver()
        } else {
            &never
        };

        loop {
            match self.command_receiver.try_recv() {
                Ok(WorkerCommand::Terminate) => {
                    break;
                }
                _ => {}
            }

            if let Ok(job) = scan_receiver.try_recv() {
                self.process_scan_job(job)?;
                continue;
            }

            select! {
                recv(self.command_receiver) -> command => {
                    match command? {
                        WorkerCommand::Terminate => {
                            break;
                        }
                    }
                }
                recv(scan_receiver) -> job => {
                    self.process_scan_job(job?)?;
                }
                recv(delete_receiver) -> job => {
                    self.process_delete_job(job?)?;
                }
            }
        }

        debug!("[{:02}] exiting", self.id);

        Ok(())
    }

    fn process_scan_job(&self, job: ScanJob) -> Result<()> {
        let file = job.file();

        trace!(
            "[{:02}] receive_scan: {}",
            self.id,
            file.lock().path().display()
        );

        let children = File::scan_children(&file)?;
        if children.len() == 0 {
            self.queue_delete_job(DeleteJob::new(file))?;
        } else {
            for child in children {
                if child.is_dir() {
                    self.queue_scan_job(ScanJob::new(Arc::new(Mutex::new(child))))?;
                    self.job_progress_store.increment_dir_found();
                } else {
                    self.queue_delete_job(DeleteJob::new(Arc::new(Mutex::new(child))))?;
                    self.job_progress_store.increment_file_found();
                }
            }
        }

        Ok(())
    }

    fn process_delete_job(&self, job: DeleteJob) -> Result<()> {
        let file = job.file();

        trace!(
            "[{:02}] receive_delete: {}",
            self.id,
            file.lock().path().display()
        );

        let file = file.lock();
        file.delete()?;

        if file.is_dir() {
            self.job_progress_store.increment_dir_deleted();
        } else {
            self.job_progress_store.increment_file_deleted();
        }

        match file.parent() {
            Some(parent) => {
                if parent.lock().children_count() == Some(0) {
                    self.queue_delete_job(DeleteJob::new(parent.clone()))?;
                }
            }
            None => {
                self.event_sender.send(Event::DeleteComplete)?;
            }
        }

        Ok(())
    }

    fn queue_scan_job(&self, job: ScanJob) -> Result<()> {
        self.job_queue.scan_sender().send(job)?;
        Ok(())
    }

    fn queue_delete_job(&self, job: DeleteJob) -> Result<()> {
        self.job_queue.delete_sender().send(job)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct StartWorkerOption {
    pub allow_delete: bool,
    pub job_queue: JobQueue,
    pub job_progress_store: JobProgressStore,
}

#[derive(Debug, Clone, Copy)]
pub enum WorkerCommand {
    Terminate,
}

pub struct WorkerPool {
    available_id: usize,
    command_senders: Vec<channel::Sender<WorkerCommand>>,
    event_sender: channel::Sender<Event>,
    event_receiver: channel::Receiver<Event>,
}

impl WorkerPool {
    pub fn new() -> WorkerPool {
        let (event_sender, event_receiver) = channel::unbounded();

        WorkerPool {
            available_id: 0,
            command_senders: vec![],
            event_sender,
            event_receiver,
        }
    }

    pub fn start_worker(&mut self, option: StartWorkerOption) {
        let (command_sender, command_receiver) = channel::unbounded();
        self.command_senders.push(command_sender);

        let worker = Worker {
            id: self.available_id,
            allow_delete: option.allow_delete,
            job_queue: option.job_queue,
            job_progress_store: option.job_progress_store,
            command_receiver,
            event_sender: self.event_sender.clone(),
        };

        thread::spawn(move || {
            worker.work_loop().unwrap();
        });

        self.available_id += 1;
    }

    pub fn terminate_workers(&self) -> Result<()> {
        debug!("[main] terminating workers");

        self.send_command(WorkerCommand::Terminate)?;

        Ok(())
    }

    pub fn wait_for_event(&self, timeout: Duration) -> Option<Event> {
        self.event_receiver.recv_timeout(timeout).ok()
    }

    fn send_command(&self, command: WorkerCommand) -> Result<()> {
        for sender in &self.command_senders {
            sender.send(command)?;
        }

        Ok(())
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.terminate_workers().unwrap();
    }
}
