use regex::Regex;
use std::{error, fmt, fs};
use std::fs::File;
use std::io::Read;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    BranchReadError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::BranchReadError => write!(f, "unable to read branch(es)"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BranchReadError => "unable to read branch(es)",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::BranchReadError => None,
        }
    }
}

pub fn all_branches() -> Result<Vec<String>, Error> {
    let branch_paths = try!(fs::read_dir(".git/refs/heads").map_err(|_| Error::BranchReadError));

    let mut branch_names = Vec::new();
    for branch_path in branch_paths {
        let branch_path = try!(branch_path.map_err(|_| Error::BranchReadError));
        let name = try!(
            branch_path
            .path()
            .file_name()
            .and_then(|fname| fname.to_str())
            .map(|fname| fname.to_string())
            .ok_or(Error::BranchReadError));
        branch_names.push(name.to_string());
    }

    Ok(branch_names)
}

pub fn current_branch() -> Result<String, Error> {
    lazy_static! {
        static ref SYMBOLIC_REF_REGEX: Regex =
            Regex::new(r"^ref: refs/heads/(?P<branch>.+)\s*$").unwrap();
    }

    let mut head_file = try!(File::open(".git/HEAD").map_err(|_| Error::BranchReadError));
    let mut head_contents = String::new();
    try!(head_file.read_to_string(&mut head_contents).map_err(|_| Error::BranchReadError));

    let caps = try!(SYMBOLIC_REF_REGEX.captures(&head_contents).ok_or(Error::BranchReadError));
    Ok(caps["branch"].to_string())
}
