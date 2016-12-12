//! Provides any functionality related to specifying and resolving revisions that name specific
//! objects. See gitrevisions(7) for the full specification on how revisions are specified, of
//! which this module will provide a subset.

use std::{error, fmt, fs};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use regex::Regex;
use commits;
use objects;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Currently the only error is a generic "this revision is invalid" error. As we try to handle
    /// more types of revisions, we'll have more specific errors that can occur.
    InvalidRevision,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidRevision => write!(f, "invalid revision"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidRevision => "invalid revision",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::InvalidRevision => None,
        }
    }
}

fn parent_of_commit(rev: &str) -> Result<objects::Name, Error> {
    let resolved = try!(resolve(rev));
    let object =
        try!(objects::read_object(&resolved).map_err(|_| Error::InvalidRevision));

    match object {
        objects::Object::Commit(
            commits::Commit { parent: Some(parent), .. }) => return Ok(parent),
        _ => return Err(Error::InvalidRevision),
    }
}

/// Given a revision as outlined in gitrevisions(7), resolve it to a canonical, 40-byte SHA-1
/// object name. The process of resolving a revision may require going to the filesystem to look up
/// objects and refs.
pub fn resolve(rev: &str) -> Result<objects::Name, Error> {
    lazy_static! {
        static ref FULL_SHA1_REGEX: Regex = Regex::new(r"^[0-9a-f]{40}$").unwrap();
        static ref PARTIAL_SHA1_REGEX: Regex = Regex::new(r"^[0-9a-f]{4,39}$").unwrap();
        static ref ANCESTOR_REGEX: Regex = Regex::new(r"^(?P<child>.+)~(?P<num>\d+)$").unwrap();

        static ref SYMBOLIC_REF_REGEX: Regex = Regex::new(r"^ref: (?P<ref>.+)\s*$").unwrap();
    }

    if rev == "HEAD" {
        let mut head_file = try!(File::open(".git/HEAD").map_err(|_| Error::InvalidRevision));
        let mut head_contents = String::new();
        try!(head_file.read_to_string(&mut head_contents).map_err(|_| Error::InvalidRevision));

        let caps = try!(SYMBOLIC_REF_REGEX.captures(&head_contents).ok_or(Error::InvalidRevision));
        let mut ref_filename = PathBuf::from(".git");
        ref_filename.push(&caps["ref"]);

        let mut ref_file = try!(File::open(ref_filename).map_err(|_| Error::InvalidRevision));
        let mut ref_contents = String::new();
        try!(ref_file.read_to_string(&mut ref_contents).map_err(|_| Error::InvalidRevision));

        return Ok(objects::Name(ref_contents.trim().to_string()));
    } else if rev.ends_with("^") {
        let child = &rev[..(rev.len() - 1)];
        return parent_of_commit(child);
    } else if FULL_SHA1_REGEX.is_match(rev) {
        return Ok(objects::Name(rev.to_string()));
    } else if PARTIAL_SHA1_REGEX.is_match(rev) {
        let prefix = &rev[..2];
        let suffix = &rev[2..];

        let dir = format!(".git/objects/{}", prefix);
        let files = try!(fs::read_dir(dir).map_err(|_| Error::InvalidRevision));

        let mut matching_files = Vec::new();
        for file in files {
            let filename = try!(file.map_err(|_| Error::InvalidRevision)).file_name();
            let filename = try!(filename.into_string().map_err(|_| Error::InvalidRevision));
            if filename.starts_with(suffix) {
                matching_files.push(filename);
            }
        }

        if matching_files.is_empty() {
            return Err(Error::InvalidRevision);
        }

        // Because we don't have an example of an ambiguous four-character SHA1, we'll ignore that
        // case until we find such a partial SHA1.
        assert!(matching_files.len() == 1);

        let full_sha1 = format!("{}{}", prefix, matching_files[0]);
        return Ok(objects::Name(full_sha1));
    } else if ANCESTOR_REGEX.is_match(rev) {
        let caps = try!(ANCESTOR_REGEX.captures(rev).ok_or(Error::InvalidRevision));
        let num = try!(caps["num"].parse::<u64>().map_err(|_| Error::InvalidRevision));

        let child = try!(resolve(&caps["child"]));
        let mut parent = child;
        for _ in 0..num {
            let objects::Name(name) = parent;
            parent = try!(parent_of_commit(&name));
        }

        return Ok(parent);
    } else {
        let mut ref_filename = PathBuf::from(".git/refs/heads");
        ref_filename.push(rev);

        let mut file = try!(File::open(ref_filename).map_err(|_| Error::InvalidRevision));

        let mut contents = String::new();
        try!(file.read_to_string(&mut contents).map_err(|_| Error::InvalidRevision));

        return Ok(objects::Name(contents.trim().to_string()));
    }
}
