/// Provides functionality for reading writing the objects that make up the content-addressable
/// database that is git.

use flate2::read::ZlibDecoder;
use std::env;
use std::error;
use std::error::Error as StdError;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::Read;
use std::path;
use std::str;

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
    InvalidFile(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IOError(ref err) => write!(f, "IO error: {}", err),
            Error::InvalidFile(ref description) => write!(f, "invalid file: {}", description),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IOError(ref err) => err.description(),
            Error::InvalidFile(ref description) => description,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IOError(ref err) => Some(err),
            ref err => Some(err)
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

fn invalid_header_error() -> Error {
    Error::InvalidFile("unable to read header".to_string())
}

fn std_error_to_objects_error<T>(e: T) -> Error
        where T: error::Error {
    Error::InvalidFile(e.description().to_string())
}

fn read_until(contents: &Vec<u8>, until: char) -> Result<(&str, Vec<u8>), Error> {
    let mut split = contents.splitn(2, |c| (*c as char) == until);

    let read = try!(split.next().ok_or(invalid_header_error()));
    let parsed = try!(str::from_utf8(read).map_err(std_error_to_objects_error));

    let rest = try!(split.next().ok_or(invalid_header_error()));

    Ok((parsed, rest.to_vec()))
}

fn read_type(contents: &Vec<u8>) -> Result<(Type, Vec<u8>), Error> {
    let (type_str, rest) = try!(read_until(contents, ' '));

    let object_type = try!(match type_str {
        "blob" => Ok(Type::Blob),
        "tree" => Ok(Type::Tree),
        "commit" => Ok(Type::Commit),
        value => Err(Error::InvalidFile(format!("invalid type: {}", value))),
    });

    Ok((object_type, rest.to_vec()))
}

fn read_size(contents: &Vec<u8>) -> Result<(u64, Vec<u8>), Error> {
    let (size_str, rest) = try!(read_until(contents, '\0'));

    let size = try!(size_str.parse::<u64>()
                    .map_err(|e| Error::InvalidFile(e.description().to_string())));

    Ok((size, rest.to_vec()))
}

pub fn read_header(name: &ObjectName) -> Result<Header, Error> {
    let path = try!(get_object_path(name));

    // read file
    let file = try!(File::open(path.as_path()).map_err(|e| Error::IOError(e)));
    let mut decoder = ZlibDecoder::new(file);
    let mut buffer = Vec::new();
    try!(decoder.read_to_end(&mut buffer).map_err(|e| Error::IOError(e)));

    let (object_type, rest) = try!(read_type(&buffer));
    let (size, _) = try!(read_size(&rest));
    Ok(Header { object_type: object_type, content_length: size })
}
