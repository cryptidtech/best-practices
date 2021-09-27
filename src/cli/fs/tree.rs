use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    pub(crate) static ref EMPTY_PATHBUF: PathBuf = {
        PathBuf::new()
    };
}

#[derive(Clone)]
pub(crate) enum TreeWork {
    Scan(PathBuf),
    Digest(PathBuf)
}


