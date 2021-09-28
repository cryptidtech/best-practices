use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    pub(crate) static ref EMPTY_PATHBUF: PathBuf = {
        PathBuf::new()
    };
}

pub mod treeitem;
pub mod treelist;
pub mod treeindex;
pub use treeitem::*;
pub use treelist::*;
pub use treeindex::*;
