use crate::file::File;
use crossbeam::channel;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug)]
pub enum JobType {
    Scan,
    Delete,
}

#[derive(Debug, Clone)]
pub struct JobPool {
    pub scan_sender: channel::Sender<Arc<Mutex<File>>>,
    pub scan_receiver: channel::Receiver<Arc<Mutex<File>>>,
    pub delete_sender: channel::Sender<Arc<Mutex<File>>>,
    pub delete_receiver: channel::Receiver<Arc<Mutex<File>>>,
}

impl JobPool {
    pub fn new() -> Self {
        let (scan_sender, scan_receiver) = channel::unbounded();
        let (delete_sender, delete_receiver) = channel::unbounded();

        JobPool {
            scan_sender,
            scan_receiver,
            delete_sender,
            delete_receiver,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JobProgress {
    found_dirs: Arc<Mutex<usize>>,
    found_files: Arc<Mutex<usize>>,
    deleted_dirs: Arc<Mutex<usize>>,
    deleted_files: Arc<Mutex<usize>>,
    pub event_sender: channel::Sender<JobProgressEvent>,
    pub event_receiver: channel::Receiver<JobProgressEvent>,
}

impl JobProgress {
    pub fn new() -> Self {
        let (sender, receiver) = channel::unbounded();

        JobProgress {
            found_dirs: Arc::new(Mutex::new(0usize)),
            found_files: Arc::new(Mutex::new(0usize)),
            deleted_dirs: Arc::new(Mutex::new(0usize)),
            deleted_files: Arc::new(Mutex::new(0usize)),
            event_sender: sender,
            event_receiver: receiver,
        }
    }

    pub fn report_dir_found(&self) {
        let mut found_dirs = self.found_dirs.lock();
        *found_dirs += 1;
    }

    pub fn report_file_found(&self) {
        let mut found_files = self.found_files.lock();
        *found_files += 1;
    }

    pub fn report_dir_deleted(&self) {
        let mut deleted_dirs = self.deleted_dirs.lock();
        *deleted_dirs += 1;
    }

    pub fn report_file_deleted(&self) {
        let mut deleted_files = self.deleted_files.lock();
        *deleted_files += 1;
    }

    pub fn print_progress(&self) {
        let found_dirs = *self.found_dirs.lock();
        let found_files = *self.found_files.lock();
        let deleted_dirs = *self.deleted_dirs.lock();
        let deleted_files = *self.deleted_files.lock();

        println!("  {:<7}  {:>10}  {:>10}", "", "Folders", "Files");
        println!("  {:<7}  {:>10}  {:>10}", "Found", found_dirs, found_files);
        println!(
            "  {:<7}  {:>10}  {:>10}",
            "Deleted", deleted_dirs, deleted_files
        );
    }
}

#[derive(PartialOrd, PartialEq, Debug)]
pub enum JobProgressEvent {
    DeleteComplete,
}
