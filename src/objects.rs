/// Provides functionality for reading writing the objects that make up the content-addressable
/// database that is git.

use flate2::read::ZlibDecoder;
use regex::Regex;
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path;
use std::str;

/// An object name, which must be a 40-byte hexadecimal string containing the SHA-1 of the object
/// being referenced. It is expected that such an object name is constructed either when the object
/// is first being written, or by resolving a reference or revision.
#[derive(Debug, PartialEq, Eq)]
pub struct Name(pub String);

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Name(ref value) = *self;
        write!(f, "objects::Name({})", value)
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

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IOError(ref err) => err.description(),
            Error::InvalidFile(ref description) => description,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::IOError(ref err) => Some(err),
            ref err => Some(err)
        }
    }
}

fn get_object_path(name: &Name) -> Result<path::PathBuf, Error> {
    let cwd = try!(env::current_dir().map_err(|e| Error::IOError(e)));
    let Name(ref sha1) = *name;
    let (dir, file) = sha1.split_at(2);
    Ok(cwd
       .join(".git/objects")
       .join(dir)
       .join(file))
}

fn std_error_to_objects_error<T>(e: T) -> Error
        where T: StdError {
    Error::InvalidFile(e.description().to_string())
}

fn read_until<R>(mut reader: &mut R, until: char) -> Result<String, Error>
        where R: BufRead, {
    let mut buffer = Vec::new();
    try!(reader.read_until(until as u8, &mut buffer).map_err(std_error_to_objects_error));

    // The last character is the one that we're reading up to, so discard that before processing
    // the bytes that were read.
    buffer.pop();

    str::from_utf8(&buffer)
        .map(|s| s.to_string())
        .map_err(std_error_to_objects_error)
}

fn read_type<R>(mut reader: &mut R) -> Result<Type, Error>
        where R: BufRead, {
    let type_str = try!(read_until(&mut reader, ' '));

    let object_type = try!(match type_str.as_ref() {
        "blob" => Ok(Type::Blob),
        "tree" => Ok(Type::Tree),
        "commit" => Ok(Type::Commit),
        value => Err(Error::InvalidFile(format!("invalid type: {}", value))),
    });

    Ok(object_type)
}

fn read_size<R>(mut reader: &mut R) -> Result<u64, Error>
        where R: BufRead, {
    let mut buffer = Vec::new();
    try!(reader.read_until('\0' as u8, &mut buffer).map_err(std_error_to_objects_error));
    buffer.pop();
    let size_str = try!(str::from_utf8(&buffer).map_err(std_error_to_objects_error));
    let size = try!(size_str.parse::<u64>()
                    .map_err(|e| Error::InvalidFile(e.description().to_string())));

    Ok(size)
}

fn read_file(name: &Name) -> Result<BufReader<ZlibDecoder<File>>, Error> {
    let path = try!(get_object_path(name));

    // read file
    let file = try!(File::open(path.as_path()).map_err(|e| Error::IOError(e)));
    Ok(BufReader::new(ZlibDecoder::new(file)))
}

fn read_header_from_reader<R>(mut reader: &mut R) -> Result<Header, Error>
        where R: BufRead {
    let object_type = try!(read_type(&mut reader));
    let size = try!(read_size(&mut reader));
    Ok(Header { object_type: object_type, content_length: size })
}

pub fn read_header(name: &Name) -> Result<Header, Error> {
    let mut reader = try!(read_file(name));
    read_header_from_reader(&mut reader)
}

fn parse_commit<R>(mut reader: &mut R) -> Result<(), Error>
        where R: BufRead {
    lazy_static! {
        static ref TREE_REGEX: Regex = Regex::new(r"^tree (?P<rev>[0-9a-f]{40})$").unwrap();
        static ref PARENT_REGEX: Regex = Regex::new(r"^parent (?P<rev>[0-9a-f]{40})$").unwrap();
        static ref AUTHOR_REGEX: Regex =
            Regex::new(r"^author (?P<name>.+) (?P<date>\d+ [+-]\d{4})$").unwrap();
        static ref COMMITTER_REGEX: Regex =
            Regex::new(r"^committer (?P<name>.+) (?P<date>\d+ [+-]\d{4})$").unwrap();
    }

    loop {
        let mut line = String::new();
        try!(reader.read_line(&mut line).map_err(std_error_to_objects_error));

        line.pop();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // Empty line, so we're ready to read the commit message at this point.
            break;
        }

        let caps = TREE_REGEX.captures(&line);
        if caps.is_some() {
            let caps = caps.unwrap();
            let tree = Name(caps["rev"].to_string());
            println!("found tree: {}", tree);
            continue;
        }

        let caps = PARENT_REGEX.captures(&line);
        if caps.is_some() {
            let caps = caps.unwrap();
            let parent = Name(caps["rev"].to_string());
            println!("found parent: {}", parent);
            continue;
        }

        let caps = AUTHOR_REGEX.captures(&line);
        if caps.is_some() {
            let caps = caps.unwrap();
            let name = caps["name"].to_string();
            let date = caps["date"].to_string();
            println!("found author: '{}' (authored at {})", name, date);
            continue;
        }

        let caps = COMMITTER_REGEX.captures(&line);
        if caps.is_some() {
            let caps = caps.unwrap();
            let name = caps["name"].to_string();
            let date = caps["date"].to_string();
            println!("found committer: '{}' (committed at {})", name, date);
            continue;
        }

        return Err(Error::InvalidFile(format!("Unexpected line in commit object: '{}'", line)));
    }

    let mut message = String::new();
    try!(reader.read_to_string(&mut message).map_err(std_error_to_objects_error));
    message.pop();
    let message = message.trim();

    println!("found commit message:\n{}", message);
    println!("DONE");

    Ok(())
}

// TODO: return parsed object
pub fn read_object(name: &Name) -> Result<(), Error> {
    let mut reader = try!(read_file(name));
    let header = try!(read_header_from_reader(&mut reader));

    match header.object_type {
        Type::Commit => parse_commit(&mut reader),
        typ => Err(Error::InvalidFile(format!("unhandled object type: {:?}", typ)))
    }
}
