//! Provides functionality for reading/writing commit objects.

use objects::Name;

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use regex::Regex;

use std::fmt;
use std::error::Error as StdError;
use std::io::BufRead;

#[derive(Debug)]
pub enum Error {
    InvalidCommit(String),
    MissingField(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidCommit(ref description) => write!(f, "invalid commit: {}", description),
            Error::MissingField(ref field) => write!(f, "missing commit field: {}", field),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidCommit(ref description) => description,
            Error::MissingField(ref field) => field,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}

pub type CommitDateTime = DateTime<FixedOffset>;

pub struct CommitUser {
    pub name: String,
    pub date: CommitDateTime,
}

pub struct Commit {
    pub name: Name,
    pub tree: Name,
    pub parent: Option<Name>,
    pub author: CommitUser,
    pub committer: CommitUser,
    pub message: String,
}

struct CommitBuilder {
    name: Name,
    tree: Option<Name>,
    parent: Option<Name>,
    author: Option<CommitUser>,
    committer: Option<CommitUser>,
    message: Option<String>,
}

impl CommitBuilder {
    pub fn new(name: &Name) -> CommitBuilder {
        CommitBuilder {
            name: (*name).to_owned(),
            tree: None,
            parent: None,
            author: None,
            committer: None,
            message: None,
        }
    }

    pub fn tree(&mut self, tree: String) -> &mut CommitBuilder {
        self.tree = Some(Name(tree));
        self
    }

    pub fn parent(&mut self, parent: String) -> &mut CommitBuilder {
        self.parent = Some(Name(parent));
        self
    }

    pub fn author(&mut self, name: String, date: CommitDateTime) -> &mut CommitBuilder {
        self.author = Some(CommitUser { name: name, date: date });
        self
    }

    pub fn committer(&mut self, name: String, date: CommitDateTime) -> &mut CommitBuilder {
        self.committer = Some(CommitUser { name: name, date: date });
        self
    }

    pub fn message(&mut self, message: String) -> &mut CommitBuilder {
        self.message = Some(message);
        self
    }

    pub fn build(self) -> Result<Commit, Error> {
        if self.tree.is_none() {
            Err(Error::MissingField("tree".to_string()))
        } else if self.committer.is_none() {
            Err(Error::MissingField("committer".to_string()))
        } else if self.author.is_none() {
            Err(Error::MissingField("author".to_string()))
        } else if self.message.is_none() {
            Err(Error::MissingField("message".to_string()))
        } else {
            Ok(Commit {
                name: self.name,
                tree: self.tree.unwrap(),
                parent: self.parent,
                committer: self.committer.unwrap(),
                author: self.author.unwrap(),
                message: self.message.unwrap(),
            })
        }
    }
}

fn std_error_to_objects_error<T>(e: T) -> Error
        where T: StdError {
    Error::InvalidCommit(e.description().to_string())
}

fn parse_commit_date(date_str: String) -> Result<CommitDateTime, Error> {
    lazy_static! {
        static ref DATETIME_REGEX: Regex =
            Regex::new(concat!(
                r"^(?P<timestamp>[0-9][0-9]*) ",
                r"(?P<tz_hours>[+-][0-9]{2})",
                r"(?P<tz_minutes>[0-9]{2})")).unwrap();
    }

    let caps = try!(DATETIME_REGEX.captures(&date_str)
                    .ok_or(Error::InvalidCommit(format!("invalid date: {}", date_str))));

    let utc = try!(
        caps["timestamp"].parse::<i64>()
        .map_err(std_error_to_objects_error)
        .and_then(|t| NaiveDateTime::from_timestamp_opt(t, 0)
                  .ok_or(Error::InvalidCommit(format!("invalid timestamp: {}", date_str)))));

    let tz_hours = try!(caps["tz_hours"].parse::<i32>().map_err(std_error_to_objects_error));
    let tz_minutes = try!(caps["tz_minutes"].parse::<i32>().map_err(std_error_to_objects_error));
    let tz = try!(FixedOffset::east_opt(tz_hours * 3600 + tz_minutes * 60)
                  .ok_or(Error::InvalidCommit(format!("invalid timezone: {}", date_str))));

    Ok(DateTime::from_utc(utc, tz))
}

pub fn parse_commit<R>(mut reader: &mut R, name: &Name) -> Result<Commit, Error>
        where R: BufRead {
    lazy_static! {
        static ref TREE_REGEX: Regex = Regex::new(r"^tree (?P<rev>[0-9a-f]{40})$").unwrap();
        static ref PARENT_REGEX: Regex = Regex::new(r"^parent (?P<rev>[0-9a-f]{40})$").unwrap();
        static ref AUTHOR_REGEX: Regex =
            Regex::new(r"^author (?P<name>.+) (?P<date>\d+ [+-]\d{4})$").unwrap();
        static ref COMMITTER_REGEX: Regex =
            Regex::new(r"^committer (?P<name>.+) (?P<date>\d+ [+-]\d{4})$").unwrap();
    }

    let mut commit_builder = CommitBuilder::new(name);
    let mut line = String::new();
    loop {
        line.clear();
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
            let tree = caps["rev"].to_string();
            commit_builder.tree(tree);
            continue;
        }

        let caps = PARENT_REGEX.captures(&line);
        if caps.is_some() {
            let caps = caps.unwrap();
            let parent = caps["rev"].to_string();
            commit_builder.parent(parent);
            continue;
        }

        let caps = AUTHOR_REGEX.captures(&line);
        if caps.is_some() {
            let caps = caps.unwrap();
            let name = caps["name"].to_string();
            let date = try!(parse_commit_date(caps["date"].to_string()));
            commit_builder.author(name, date);
            continue;
        }

        let caps = COMMITTER_REGEX.captures(&line);
        if caps.is_some() {
            let caps = caps.unwrap();
            let name = caps["name"].to_string();
            let date = try!(parse_commit_date(caps["date"].to_string()));
            commit_builder.committer(name, date);
            continue;
        }

        return Err(Error::InvalidCommit(format!("Unexpected line in commit object: '{}'", line)));
    }

    let mut message = String::new();
    try!(reader.read_to_string(&mut message).map_err(std_error_to_objects_error));
    message.pop();
    commit_builder.message(message.trim().to_string());

    commit_builder.build()
}
