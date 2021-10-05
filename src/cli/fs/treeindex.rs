use crate::{
    error::Error,
    Result,
    cli::fs::{
        TreeItemBuilder,
        TreeItemDupes,
        TreeList
    }
};
use log::debug;
use std::collections::HashMap;
use std::convert::From;
use std::ffi::OsString;
use std::fs;
use std::io::{BufReader, BufRead, Read};
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) enum TreeWork {
    Scan(PathBuf),
    Digest(PathBuf)
}

// A TreeIndex is a map from digest to TreeItemDupes
#[derive(Clone, Default)]
pub struct TreeIndex {
    pub idx: HashMap<String, TreeItemDupes>
}

impl TreeIndex {

    pub fn max(&self) -> u64 {
        let mut max = 0;
        for (_, v) in self.idx.iter() {
            if v.item.size > max {
                max = v.item.size;
            }
        }
        max
    }

    pub fn count_dupes(&self) -> usize {
        let mut count = 0;
        for (_, v) in self.idx.iter() {
            count += v.dupes.len();
        }
        count
    }
}

enum TreeIndexFrom<'a> {
    New,
    List(&'a TreeList),
    Reader(&'a mut Box<dyn Read>),
    Confirm(&'a TreeIndex)
}

impl<'a> Default for TreeIndexFrom<'a> {
    fn default() -> Self {
        TreeIndexFrom::New
    }
}

#[derive(Default)]
pub struct TreeIndexBuilder<'a> {
    with_dupes: bool,
    from: TreeIndexFrom<'a>,
}


impl<'a> TreeIndexBuilder<'a> {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dupes(mut self, dupes: bool) -> Self {
        self.with_dupes = dupes;
        self
    }

    pub fn from_list(mut self, list: &'a TreeList) -> Self {
        self.from = TreeIndexFrom::List(list);
        self
    }

    pub fn from_reader(mut self, r: &'a mut Box<dyn Read>) -> Self {
        self.from = TreeIndexFrom::Reader(r);
        self
    }

    pub fn confirm(mut self, index: &'a TreeIndex) -> Self {
        self.from = TreeIndexFrom::Confirm(index);
        self
    }

    pub fn build(self) -> Result<TreeIndex> {
        let mut ti = TreeIndex::default();
        match self.from {

            // do nothing
            TreeIndexFrom::New => {}

            // build an index from a tree list
            TreeIndexFrom::List(l) => {
                debug!("constructing index from list");
                for i in &l.list {
                    match ti.idx.get_mut(&i.digest) {
                        Some(item) => {
                            if self.with_dupes {
                                item.push(i.path.clone());
                            }
                        },
                        None => {
                            ti.idx.insert(i.digest.clone(), TreeItemDupes::from(i));
                        }
                    }
                }
            },

            TreeIndexFrom::Reader(r) => {
                debug!("constructing index from reader");
                let r = BufReader::new(r);
                let mut last_digest = "-".to_string();

                let mut line_count = 0;
                for line in r.lines() {
                    line_count += 1;
                    let mut line = line.unwrap();

                    // read the digest
                    let mut digest = match line.find(char::is_whitespace) {
                        Some(idx) => {
                            let rest = line.split_off(idx);
                            let d = line.clone();
                            line = rest[1..].to_string();
                            d
                        },
                        None => return Err(Error::InvalidFormat(format!("missing digest on line {}", line_count)))
                    };

                    // if this is NOT a dupe line, read the file size
                    let size = {
                        if digest != "-" {
                            match line.find(char::is_whitespace) {
                                Some(idx) => {
                                    let rest = line.split_off(idx);
                                    let s = line.parse::<u64>().unwrap_or(0u64);
                                    line = rest[1..].to_string();
                                    s
                                },
                                None => return Err(Error::InvalidFormat(format!("missing size on line {}", line_count)))
                            }
                        } else {
                            0u64
                        }
                    };

                    if digest == "-" {
                        digest = last_digest.clone();
                    } else {
                        last_digest = digest.clone();
                    }

                    let path = Rc::new(PathBuf::from(OsString::from(line)));

                    // look up the digest
                    match ti.idx.get_mut(&digest) {
                        Some(item) => {
                            if self.with_dupes {
                                item.push(path)
                            }
                        },
                        None => {
                            ti.idx.insert(digest.clone(), TreeItemDupes::new(&digest, &path, size));
                        }
                    }
                }
            },

            TreeIndexFrom::Confirm(i) => {
                debug!("constructing confirmed dupe index from index");
                for (d, i) in i.idx.iter() {

                    // do a full digest of the file
                    let item = TreeItemBuilder::new()
                        .fast(false)
                        .path(&i.item.path)
                        .build()?;

                    // add it to the index
                    ti.idx.insert(d.to_string(), TreeItemDupes::from(&item));

                    // go through each of the dupes and do full digests on them to confirm
                    // they truly are matches
                    for p in &i.dupes {

                        // confirm the size and use that
                        let size = match fs::metadata(&p.as_path()) {
                            Ok(meta) => meta.len(),
                            Err(_) => 0u64
                        };

                        if size == i.item.size {
                            let dupe = TreeItemBuilder::new()
                                .fast(false)
                                .path(&p)
                                .build()?;

                            // if there is a match, then the match is confirmed and we
                            // add it as a dupe, otherwise we do nothing
                            match ti.idx.get_mut(&dupe.digest) {
                                Some(item) => {
                                    debug!("confirmed dupe {} {}", i.item.path.to_string_lossy(), dupe.path.to_string_lossy());
                                    item.push(dupe.path.clone());
                                },
                                None => {
                                    debug!("invalid dupe {} {}", i.item.path.to_string_lossy(), dupe.path.to_string_lossy());
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(ti)
    }
}


