use crate::Result;
use std::fs::File;
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

///! This function takes an optional path and returns a concrete Read'er object.
///! This is most useful for command line applications that take either a file
///! or stdin as input. The user can specify "-" or nothing and the result of
///! this function is a Read'er for the stdin stream. If they specify a file,
///! then the Read'er is the file stream. If there is an error opening the file
///! then a crate::error::IoError result.
pub fn reader(path: &Option<PathBuf>) -> Result<Box<dyn Read>> {
    match path {
        Some(p) => {
            if p.to_string_lossy() == "-" {
                Ok(Box::new(io::stdin()) as Box<dyn Read>)
            } else {
                let path = Path::new(&p);
                Ok(Box::new(File::open(&path)?) as Box<dyn Read>)
            }
        }
        None => Ok(Box::new(io::stdin()) as Box<dyn Read>)
    }
}

///! This function takes an optional path and returns a concrete Read'er object.
///! This is most useful for command line applications that take either a file
///! or stdin as input. The user can specify "-" or nothing and the result of
///! this function is a Read'er for the stdin stream. If they specify a file,
///! then the Read'er is the file stream. If there is an error opening the file
///! then a crate::error::IoError result. Secure read implies whatever the
///! types is not echoed back to the TTY.
pub fn secure_reader(path: &Option<PathBuf>) -> Result<Box<dyn Read>> {
    match path {
        Some(p) => {
            if p.to_string_lossy() == "-" {
                let secret = rpassword::prompt_password("")?;
                Ok(Box::new(io::Cursor::new(secret)))
            } else {
                let path = Path::new(&p);
                Ok(Box::new(File::open(&path)?) as Box<dyn Read>)
            }
        }
        None => {
            let secret = rpassword::prompt_password("")?;
            Ok(Box::new(io::Cursor::new(secret)))
        }
    }
}

///! This function works in tandem with the above reader function except that
///! it returns a convenient OsString name for the reader. This is used for
///! verbose output to describe where the input is coming from.
pub fn reader_name(path: &Option<PathBuf>) -> Result<OsString> {
    match path {
        Some(p) => {
            if p.to_string_lossy() == "-" {
                Ok(OsString::from("stdin"))
            } else {
                Ok(p.clone().into_os_string())
            }
        }
        None => Ok(OsString::from("stdin"))
    }
}

///! This function works the same as the reader function but is for writers.
///! If the path is provided then the Write'er is for the file stream. If the
///! path is not provided then the Write'er is for the stdout stream.
pub fn writer(path: &Option<PathBuf>) -> Result<Box<dyn Write>> {
    match path {
        Some(p) => {
            let path = Path::new(&p);
            Ok(Box::new(File::create(&path)?) as Box<dyn Write>)
        }
        None => Ok(Box::new(io::stdout()) as Box<dyn Write>)
    }
}

///! This function gives the name for the writer for verbose output purposes.
pub fn writer_name(path: &Option<PathBuf>) -> Result<OsString> {
    match path {
        Some(p) => {
            Ok(p.clone().into_os_string())
        }
        None => Ok(OsString::from("stdout"))
    }
}

///! This function takes an optional path and returns the path if supplied,
///! otherwise it defaults to the current working directory.
pub fn dir(path: &Option<PathBuf>) -> Result<PathBuf> {
    match path {
        Some(p) => Ok(p.to_path_buf()),
        None => Ok(std::env::current_dir()?)
    }
}

///! This function works with the above dir function but gives the name of the
///! directory for verbose output purposes.
pub fn dir_name(path: &Option<PathBuf>) -> Result<OsString> {
    match path {
        Some(p) => {
            Ok(p.clone().into_os_string())
        }
        None => Ok(OsString::from("pwd"))
    }
}

