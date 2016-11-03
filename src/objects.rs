/// Provides functionality for reading writing the objects that make up the content-addressable
/// database that is git.

use std::env;
use std::error;
use std::fmt;
use std::io;
use std::path;

/// An object name, which must be a 40-byte hexadecimal string containing the SHA-1 of the object
/// being referenced. It is expected that such an object name is constructed either when the object
/// is first being written, or by resolving a reference or revision.
#[derive(Debug, PartialEq, Eq)]
pub struct ObjectName(pub String);

impl fmt::Display for ObjectName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ObjectName(ref value) = *self;
        write!(f, "ObjectName({})", value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    Blob,
    Tree,
    Commit,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub object_type: Type,
    pub content_length: u64,
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IOError(ref err) => write!(f, "IO error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IOError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IOError(ref err) => Some(err),
        }
    }
}

fn get_object_path(name: &ObjectName) -> Result<path::PathBuf, Error> {
    let cwd = try!(env::current_dir().map_err(|e| Error::IOError(e)));
    let ObjectName(ref sha1) = *name;
    let (dir, file) = sha1.split_at(2);
    Ok(cwd
       .join(".git/objects")
       .join(dir)
       .join(file))
}

pub fn read_header(name: &ObjectName) -> Result<Header, Error> {
    let path = get_object_path(name);
    println!("path: {:?}", path);
    Ok(Header { object_type: Type::Blob, content_length: 100 })
}
