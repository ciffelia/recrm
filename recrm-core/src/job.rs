use crate::file::File;
use crossbeam::channel;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct JobQueue {
    pub scan_sender: channel::Sender<Arc<Mutex<File>>>,
    pub scan_receiver: channel::Receiver<Arc<Mutex<File>>>,
    pub delete_sender: channel::Sender<Arc<Mutex<File>>>,
    pub delete_receiver: channel::Receiver<Arc<Mutex<File>>>,
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
}

#[derive(Debug)]
pub struct JobProgress {
    found_dirs: usize,
    found_files: usize,
    deleted_dirs: usize,
    deleted_files: usize,
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

    pub fn get_progress(&self) -> JobProgress {
        JobProgress {
            found_dirs: self.found_dirs.load(Ordering::Relaxed),
            found_files: self.found_files.load(Ordering::Relaxed),
            deleted_dirs: self.deleted_dirs.load(Ordering::Relaxed),
            deleted_files: self.deleted_files.load(Ordering::Relaxed),
        }
    }
}
