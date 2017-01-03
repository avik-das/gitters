//! Provides functionality for reading/writing the index file, which contains a list of all the
//! files tracked by the content-addressable database that is git.

use std::collections::HashSet;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::iter::FromIterator;
use std::path::PathBuf;

use byteorder::{NetworkEndian, ReadBytesExt};
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidIndex(String),
    InvalidEntry(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidIndex(ref reason) => write!(f, "invalid index file: {}", reason),
            Error::InvalidEntry(ref reason) => write!(f, "invalid index entry: {}", reason),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidIndex(ref reason) => reason,
            Error::InvalidEntry(ref reason) => reason,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::InvalidIndex(_) => None,
            Error::InvalidEntry(_) => None,
        }
    }
}

pub struct Index {
    pub version: u32,
    pub entries: Vec<Entry>,
}

impl Index {
    pub fn read() -> Result<Index, Error> {
        let index_file =
            try!(File::open(".git/index")
                 .map_err(|_| Error::InvalidIndex("unable to read index file".to_string())));
        let mut reader = BufReader::new(index_file);

        // Read first four bytes, make sure they're DIRC
        let mut header_buffer = [0; 4];
        try!(reader.read_exact(&mut header_buffer)
             .map_err(|_| Error::InvalidIndex("unable to read header".to_string())));
        if header_buffer != [0x44, 0x49, 0x52, 0x43] {
            return Err(Error::InvalidIndex(format!("invalid header: {:?}", header_buffer)));
        }

        let version = try!(reader.read_u32::<NetworkEndian>()
                           .map_err(|err| Error::InvalidIndex(err.description().to_string())));

        let num_entries =
            try!(reader.read_u32::<NetworkEndian>()
                 .map_err(|err| Error::InvalidIndex(err.description().to_string())));
        let mut entries = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            let entry = try!(Entry::read(version, &mut reader));
            entries.push(entry);
        }

        // Deliberately ignore any extensions. Some of the extensions can have a large impact on
        // how the index file should be parsed, but for the repo tracking this project, ignoring
        // the extensions seems safe for now.

        Ok(Index {
            version: version,
            entries: entries
        })
    }
}

pub struct Entry {
    pub sha1: String,
    pub path: PathBuf,
}

impl Entry {
    fn read(version: u32, reader: &mut BufRead) -> Result<Entry, Error> {
        // We'll deliberately not read many of the fields for now, choosing to add in more
        // functionality as it is needed. For example, the big chunk of bytes at the beginning of
        // each entry is ignored for now, but it can be parsed correctly as the need arises.

        let mut entry_length = 0;

        try!(reader.read_exact(&mut [0; 40])
             .map_err(|_| Error::InvalidEntry("unable to read entry: prefix".to_string())));
        entry_length += 40;

        let mut sha1_bytes = [0; 20];
        try!(reader.read_exact(&mut sha1_bytes)
             .map_err(|_| Error::InvalidEntry("unable to read entry: sha1".to_string())));
        let sha1 = sha1_bytes
            .iter()
            .map(|n| format!("{:02x}", n))
            .collect::<Vec<_>>()
            .concat();
        entry_length += 20;

        try!(reader.read_exact(&mut [0; 2])
             .map_err(|_| Error::InvalidEntry("unable to read entry: flags".to_string())));
        entry_length += 2;

        if version >= 3 {
            try!(reader.read_exact(&mut [0; 2])
                 .map_err(|_| Error::InvalidEntry(
                         "unable to read entry: additional flags".to_string())));
            entry_length += 2;
        }

        let mut path_name_bytes = Vec::new();
        let path_name_length =
            try!(reader.read_until(0, &mut path_name_bytes)
                 .map_err(|_| Error::InvalidEntry(
                         "unable to read entry: path name".to_string())));
        path_name_bytes.pop();  // remove the null byte
        let path_name =
            try!(String::from_utf8(path_name_bytes)
                 .map_err(|_| Error::InvalidEntry(
                         "unable to read entry: path name from UTF8".to_string())));
        entry_length += path_name_length;

        let path_name_padding =
            if version >= 4 {
                // Starting with version 4, there's no longer any padding after the path name.
                0
            } else {
                (8 - (entry_length % 8)) % 8
            };
        try!(reader.read_exact(&mut vec![0; path_name_padding])
             .map_err(|_| Error::InvalidEntry(
                     "unable to read entry: path name padding".to_string())));

        let path = try!(
            fs::canonicalize(path_name.clone())
            .map_err(|_| Error::InvalidEntry(
                    format!("unable to parse path name: {}", path_name))));

        Ok(Entry {
            sha1: sha1,
            path: path,
        })
    }
}

// Currently, this function is being used solely for "git ls-files --others", so it's okay to read
// the index and walk the working directly right in this function. In the future, when this is used
// for "git status", we'll want to read the index and the working directory outside this function
// so that work can be re-used for multiple operations.
pub fn untracked_files() -> Result<Vec<PathBuf>, Error> {
    let index = try!(Index::read());
    let tracked_files: HashSet<PathBuf> =
        HashSet::from_iter(index.entries
                           .iter()
                           .map(|e| e.path.to_path_buf())
                           .collect::<Vec<_>>());

    fn is_git_dir(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s == ".git")
            .unwrap_or(false)
    }

    let all_files = WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| !is_git_dir(e))
        .filter_map(|e| e.ok())
        .filter_map(|e| fs::canonicalize(e.path()).ok())
        .filter(|e| !e.is_dir());

    let untracked = all_files
        .filter(|file| !tracked_files.contains(file));

    Ok(untracked.collect())
}
