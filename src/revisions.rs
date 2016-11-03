//! Provides any functionality related to specifying and resolving revisions that name specific
//! objects. See gitrevisions(7) for the full specification on how revisions are specified, of
//! which this module will provide a subset.

use std::error;
use std::fmt;
use regex::Regex;

#[derive(Debug)]
pub enum RevisionError {
    /// Currently the only error is a generic "this revision is invalid" error. As we try to handle
    /// more types of revisions, we'll have more specific errors that can occur.
    InvalidRevision,
}

impl fmt::Display for RevisionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RevisionError::InvalidRevision => write!(f, "invalid revision"),
        }
    }
}

impl error::Error for RevisionError {
    fn description(&self) -> &str {
        match *self {
            RevisionError::InvalidRevision => "invalid revision",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            RevisionError::InvalidRevision => None,
        }
    }
}

/// Given a revision as outlined in gitrevisions(7), resolve it to a canonical, 40-byte SHA-1
/// object name. The process of resolving a revision may require going to the filesystem to look up
/// objects and refs.
pub fn resolve(rev: &str) -> Result<&str, RevisionError> {
    lazy_static! {
        static ref FULL_SHA1_REGEX: Regex = Regex::new(r"^[0-9a-f]{40}$").unwrap();
    }

    // Currently, only support full SHA-1s, so all we have to do is check that we have the correct
    // format, and then return it.
    if FULL_SHA1_REGEX.is_match(rev) {
        return Ok(rev);
    }

    Err(RevisionError::InvalidRevision)
}
