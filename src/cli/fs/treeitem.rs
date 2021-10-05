use crate::{
    error::Error,
    Result,
    cli::fs::{
        EMPTY_PATHBUF
    }
};
use blake2b_simd::Params;
use log::debug;
use std::convert::From;
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Read};
use std::path::PathBuf;
use std::rc::Rc;

// A TreeItem is a path to a file with its digest and file size
#[derive(Clone)]
pub struct TreeItem {
    pub digest: String,
    pub path: Rc<PathBuf>,
    pub size: u64
}

impl TreeItem {
    pub fn new(digest: &str, path: &Rc<PathBuf>, size: u64) -> Self {
        Self {
            digest: digest.to_string(),
            path: path.clone(),
            size: size
        }
    }
}

impl Display for TreeItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error>
    {
        let path = match (*self.path).clone().into_os_string().into_string() {
            Ok(p) => p,
            Err(_) => return Err(std::fmt::Error)
        };
        writeln!(f, "{} {} {}", self.digest, self.size, path)?;
        Ok(())
    }
}

pub struct TreeItemBuilder<'a> {
    fast: bool,
    path: &'a PathBuf,
}

impl<'a> TreeItemBuilder<'a> {

    pub fn new() -> Self {
        TreeItemBuilder {
            fast: false,
            path: &EMPTY_PATHBUF
        }
    }

    pub fn fast(mut self, fast: bool) -> Self {
        self.fast = fast;
        self
    }

    pub fn path(mut self, path: &'a PathBuf) -> Self {
        self.path = path;
        self
    }

    pub fn build(self) -> Result<TreeItem> {
        // make sure we have a file
        if !self.path.is_file() {
            return Err(Error::NotAFile(self.path.to_path_buf()));
        }

        // get the file size
        let size = fs::metadata(&self.path)?.len();

        // open the file
        debug!("[DGST] {}", self.path.to_string_lossy());
        let mut f = File::open(self.path)?;

        // we're creating a Blake2b 32-byte digest of the file
        let mut hash = Params::new().hash_length(32).to_state();
        let mut buf = [0; 1_048_576]; // this streams a file from disk 1M at a time to hash it
        let mut num = 0;
        while num < size {
            let n = match f.read(&mut buf) {
                Ok(n) => n,
                Err(e) => {
                    debug!("failed to read data from {}", self.path.to_string_lossy());
                    return Err(Error::IoError(e));
                }
            };
            hash.update(&buf[0..n]);
            num += n as u64;

            // fast mode causes the hash to contain only the first 1 MB
            // and the last 1 MB of a file which is close enough for most
            // matching and significantly faster than hashing the whole file
            if self.fast && (num < size) && (size > 1_048_576) {
                num = match f.seek(SeekFrom::Start(size-1_048_575)) {
                    Ok(n) => n,
                    Err(e) => {
                        debug!("failed to seek to {}", size - 1_048_575);
                        return Err(Error::IoError(e));
                    }
                }
            }
        }
        let result = hash.finalize().to_hex(); // returns ArrayString<[u8; 128]>
        Ok(TreeItem::new(&result, &Rc::new(self.path.clone()), size))
    }
}

// A TreeItemDupes is a tree item with a list of paths to other files with the
// same digest as the main item
#[derive(Clone)]
pub struct TreeItemDupes {
    pub item: TreeItem,
    pub dupes: Vec<Rc<PathBuf>>
}

impl TreeItemDupes {
    pub fn new(digest: &str, path: &Rc<PathBuf>, size: u64) -> Self {
        Self {
            item: TreeItem::new(digest, path, size),
            dupes: Vec::new()
        }
    }

    pub fn push(&mut self, dupe: Rc<PathBuf>) {
        self.dupes.push(dupe);
    }
}

impl From<&TreeItem> for TreeItemDupes {
    fn from(item: &TreeItem) -> Self {
        Self {
            item: item.clone(),
            dupes: Vec::new()
        }
    }
}

impl Display for TreeItemDupes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error>
    {
        write!(f, "{}", self.item)?;
        for d in &self.dupes {
            let path = match (**d).clone().into_os_string().into_string() {
                Ok(p) => p,
                Err(_) => return Err(std::fmt::Error)
            };
            writeln!(f, "- {}", path)?;
        }
        Ok(())
    }
}


