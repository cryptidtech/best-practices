use crate::{
    Result,
    cli::fs::{
        EMPTY_PATHBUF,
        TreeItem,
        TreeItemBuilder,
        TreeWork
    },
    cli::io::dir
};
use log::debug;
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

// A TreeList is just a list of TreeItems and can contain duplicates
#[derive(Clone, Default)]
pub struct TreeList {
    pub list: Vec<TreeItem>
}

pub struct TreeListBuilder<'a> {
    fast: bool,
    max_size: u64,
    path: &'a PathBuf,
}

impl<'a> TreeListBuilder<'a> {

    pub fn new() -> Self {
        Self {
            fast: false,
            max_size: u64::MAX,
            path: &EMPTY_PATHBUF
        }
    }

    pub fn fast(mut self, fast: bool) -> Self {
        self.fast = fast;
        self
    }

    pub fn max_size(mut self, max: u64) -> Self {
        self.max_size = max;
        self
    }

    pub fn path(mut self, path: &'a PathBuf) -> Self {
        self.path = path;
        self
    }

    pub fn build(self) -> Result<TreeList> {
        // create the work queue
        let mut q: VecDeque<TreeWork> = VecDeque::new();
        q.push_back(TreeWork::Scan(dir(&Some(self.path.to_path_buf()))?));

        // create the resulting TreeList
        let mut tl = TreeList::default();

        // process the work
        while let Some(work) = q.pop_front() {
            match work {
                TreeWork::Scan(d) => {
                    debug!("[SCAN] {}", d.to_string_lossy());
                    let diter = fs::read_dir(d)?;
                    for entry in diter {
                        let entry = entry?;
                        let path = entry.path();
                        if path.is_dir() {
                            q.push_back(TreeWork::Scan(path));
                        } else if path.is_file() {
                            let size = match fs::metadata(&path) {
                                Ok(meta) => meta.len(),
                                Err(_) => 0u64
                            };
                            if size <= self.max_size {
                                q.push_back(TreeWork::Digest(path));
                            }
                        }
                    }
                },
                TreeWork::Digest(f) => {
                    tl.list.push(TreeItemBuilder::new()
                        .fast(self.fast)
                        .path(&f)
                        .build()?);
                }
            }
        }

        Ok(tl)
    }
}
