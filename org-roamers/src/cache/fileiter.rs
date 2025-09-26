use std::{
    ffi::OsStr,
    fs::{self, ReadDir},
    io,
    path::{Path, PathBuf},
};

pub struct FileIter {
    pending_dirs: Vec<ReadDir>,
}

impl FileIter {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut this = Self {
            pending_dirs: Vec::new(),
        };
        this.pending_dirs.push(fs::read_dir(path)?);
        Ok(this)
    }
}

impl Iterator for FileIter {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pending_dirs.is_empty() {
                return None;
            }

            // Process the first directory
            if let Some(entry_result) = self.pending_dirs[0].next() {
                let entry = match entry_result {
                    Ok(entry) => entry,
                    Err(e) => return Some(Err(e)),
                };

                let metadata = match entry.metadata() {
                    Ok(metadata) => metadata,
                    Err(err) => return Some(Err(err)),
                };

                if metadata.is_dir() {
                    match fs::read_dir(entry.path()) {
                        Ok(read_dir) => self.pending_dirs.push(read_dir),
                        Err(e) => return Some(Err(e)),
                    }
                }

                if metadata.is_file() && entry.path().extension() == Some(OsStr::new("org")) {
                    return Some(Ok(entry.path()));
                }
            } else {
                // Current directory is exhausted, remove it
                self.pending_dirs.remove(0);
            }
        }
    }
}
