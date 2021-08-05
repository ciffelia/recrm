use crate::file::File;
use crossbeam::channel;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct ScanJob {
    file: Arc<Mutex<File>>,
}

impl ScanJob {
    pub fn new(file: Arc<Mutex<File>>) -> Self {
        Self { file }
    }

    pub fn file(self) -> Arc<Mutex<File>> {
        self.file
    }
}

pub struct DeleteJob {
    file: Arc<Mutex<File>>,
}

impl DeleteJob {
    pub fn new(file: Arc<Mutex<File>>) -> Self {
        Self { file }
    }

    pub fn file(self) -> Arc<Mutex<File>> {
        self.file
    }
}

#[derive(Debug, Clone)]
pub struct JobQueue {
    scan_sender: channel::Sender<ScanJob>,
    scan_receiver: channel::Receiver<ScanJob>,
    delete_sender: channel::Sender<DeleteJob>,
    delete_receiver: channel::Receiver<DeleteJob>,
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

    pub fn scan_sender(&self) -> &channel::Sender<ScanJob> {
        &self.scan_sender
    }

    pub fn scan_receiver(&self) -> &channel::Receiver<ScanJob> {
        &self.scan_receiver
    }

    pub fn delete_sender(&self) -> &channel::Sender<DeleteJob> {
        &self.delete_sender
    }

    pub fn delete_receiver(&self) -> &channel::Receiver<DeleteJob> {
        &self.delete_receiver
    }
}

#[derive(Debug, Clone, Copy)]
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

    pub fn increment_dir_found(&self) {
        self.found_dirs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_file_found(&self) {
        self.found_files.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_dir_deleted(&self) {
        self.deleted_dirs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_file_deleted(&self) {
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
