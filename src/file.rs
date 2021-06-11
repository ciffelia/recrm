use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, io};

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    pub is_dir: bool,
    pub parent: Option<Arc<Mutex<Self>>>,
    pub children_count: Option<usize>,
}

impl File {
    pub fn new(path: PathBuf, parent: Option<Arc<Mutex<File>>>) -> Self {
        File {
            is_dir: path.is_dir(),
            path,
            parent,
            children_count: None,
        }
    }

    pub fn scan_children(dir: &Arc<Mutex<Self>>) -> io::Result<Vec<File>> {
        let children = fs::read_dir(&dir.lock().path)?
            .map(|res| res.map(|entry| entry.path()))
            .map(|res| res.map(|path| File::new(path, Some(dir.clone()))))
            .collect::<io::Result<Vec<File>>>();

        if let Ok(children) = &children {
            let mut dir = dir.lock();
            (*dir).children_count = Some(children.len());
        }

        children
    }

    pub fn delete(&self) -> io::Result<()> {
        if self.is_dir {
            fs::remove_dir(&self.path)?;
        } else {
            fs::remove_file(&self.path)?;
        }

        if let Some(parent) = &self.parent {
            let mut parent = parent.lock();
            (*parent).children_count = Some((*parent).children_count.unwrap() - 1)
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::time::Instant;

    #[test]
    fn enumerate_children() -> io::Result<()> {
        let root_dir = Arc::new(Mutex::new(File::new(PathBuf::from("."), None)));
        println!("{:?}", root_dir.lock());

        for entry in File::scan_children(&root_dir)? {
            println!("{:?}", entry);
        }

        Ok(())
    }

    #[test]
    fn recursively_enumerate_children() {
        let start = Instant::now();

        let root_dir = Arc::new(Mutex::new(File::new(PathBuf::from("."), None)));
        println!("{:?}", root_dir.lock());

        let mut queue = VecDeque::new();
        queue.push_back(root_dir);

        while let Some(file) = queue.pop_front() {
            println!("scanning: {:?}", file.lock());

            for child in File::scan_children(&file).unwrap() {
                println!("found: {:?}", child);
                if child.is_dir {
                    queue.push_back(Arc::new(Mutex::new(child)));
                }
            }
        }

        let end = start.elapsed();
        println!(
            "elapsed: {}.{:03}",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );
    }
}
