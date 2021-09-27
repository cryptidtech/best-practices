#[macro_use]
extern crate log;
extern crate lazy_static;

pub mod cli;
pub mod error;
pub type Result<T> = error::Result<T>;
