use anyhow::Result;
use parking_lot::Mutex;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    pub is_dir: bool,
    pub parent: Option<Arc<Mutex<File>>>,
    pub children_count: Option<usize>,
}

pub struct NewFileOptions<'a> {
    pub path: &'a dyn AsRef<Path>,
    pub is_dir: bool,
    pub parent: Option<Arc<Mutex<File>>>,
}

impl File {
    pub fn new(options: NewFileOptions) -> Self {
        File {
            path: options.path.as_ref().into(),
            is_dir: options.is_dir,
            parent: options.parent,
            children_count: None,
        }
    }

    pub fn scan_children(dir: &Arc<Mutex<Self>>) -> Result<Vec<File>> {
        let children = fs::read_dir(&dir.lock().path)?
            .map(|entry| -> Result<File> {
                let entry = entry?;
                Ok(File::new(NewFileOptions {
                    path: &entry.path(),
                    is_dir: entry.file_type()?.is_dir(),
                    parent: Some(dir.clone()),
                }))
            })
            .collect::<Result<Vec<File>>>();

        if let Ok(children) = &children {
            let mut dir = dir.lock();
            (*dir).children_count = Some(children.len());
        }

        children
    }

    pub fn delete(&self) -> Result<()> {
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
    fn enumerate_children() -> Result<()> {
        let root_dir = File::new(NewFileOptions {
            path: &".",
            is_dir: true,
            parent: None
        });
        let root_dir = Arc::new(Mutex::new(root_dir));
        println!("{:?}", root_dir.lock());

        for entry in File::scan_children(&root_dir)? {
            println!("{:?}", entry);
        }

        Ok(())
    }

    #[test]
    fn recursively_enumerate_children() {
        let start = Instant::now();

        let root_dir = File::new(NewFileOptions {
            path: &".",
            is_dir: true,
            parent: None
        });
        let root_dir = Arc::new(Mutex::new(root_dir));
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
