use crate::file::File;
use crossbeam::channel;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct JobQueue {
    scan_sender: channel::Sender<Arc<Mutex<File>>>,
    scan_receiver: channel::Receiver<Arc<Mutex<File>>>,
    delete_sender: channel::Sender<Arc<Mutex<File>>>,
    delete_receiver: channel::Receiver<Arc<Mutex<File>>>,
}

impl JobQueue {
    pub fn new() -> Self {
        let (scan_sender, scan_receiver) = channel::unbounded();
        let (delete_sender, delete_receiver) = channel::unbounded();

        JobQueue {
            scan_sender,
            scan_receiver,
            delete_sender,
            delete_receiver,
        }
    }

    pub fn scan_sender(&self) -> &channel::Sender<Arc<Mutex<File>>> {
        &self.scan_sender
    }

    pub fn scan_receiver(&self) -> &channel::Receiver<Arc<Mutex<File>>> {
        &self.scan_receiver
    }

    pub fn delete_sender(&self) -> &channel::Sender<Arc<Mutex<File>>> {
        &self.delete_sender
    }

    pub fn delete_receiver(&self) -> &channel::Receiver<Arc<Mutex<File>>> {
        &self.delete_receiver
    }
}

#[derive(Debug)]
pub struct JobProgress {
    pub found_dirs: usize,
    pub found_files: usize,
    pub deleted_dirs: usize,
    pub deleted_files: usize,
}

#[derive(Debug, Clone)]
pub struct JobProgressStore {
    found_dirs: Arc<AtomicUsize>,
    found_files: Arc<AtomicUsize>,
    deleted_dirs: Arc<AtomicUsize>,
    deleted_files: Arc<AtomicUsize>,
}

impl JobProgressStore {
    pub fn new() -> Self {
        JobProgressStore {
            found_dirs: Arc::new(AtomicUsize::new(0)),
            found_files: Arc::new(AtomicUsize::new(0)),
            deleted_dirs: Arc::new(AtomicUsize::new(0)),
            deleted_files: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn add_dir_found(&self) {
        self.found_dirs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_file_found(&self) {
        self.found_files.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_dir_deleted(&self) {
        self.deleted_dirs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_file_deleted(&self) {
        self.deleted_files.fetch_add(1, Ordering::Relaxed);
    }

    pub fn progress(&self) -> JobProgress {
        JobProgress {
            found_dirs: self.found_dirs.load(Ordering::Relaxed),
            found_files: self.found_files.load(Ordering::Relaxed),
            deleted_dirs: self.deleted_dirs.load(Ordering::Relaxed),
            deleted_files: self.deleted_files.load(Ordering::Relaxed),
        }
    }
}
