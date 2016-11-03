/// Provides functionality for reading writing the objects that make up the content-addressable
/// database that is git.

use std::fmt;

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
