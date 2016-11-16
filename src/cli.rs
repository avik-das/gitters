//! Provides utilities for creating standardized command-line interfaces (CLIs), such as a means of
//! propagating error status codes.

use std::error::Error as StdError;
use std::fmt;
use std::process;
use std::result;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub status: i32
}

impl Error {
    /// Coerce a standard `Error` into a CLI `Error`, with a status code. Private because we will
    /// typically be using `wrap_with_status` to map a standard `Result` containing an error into
    /// one containing a CLI `Error`. Can be made public if necessary in the future.
    fn from_error<T>(cause: T, status: i32) -> Error
            where T: StdError, {
        Error { message: cause.description().to_string(), status: status }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}

pub type Result = result::Result<(), Error>;

/// Coerce a `Result` containing a standard `Error` into one containing a CLI `Error` if necessary.
/// If the coercion takes place, the resulting `Error` will have the given status code attached to
/// it so the process can exit with that status code.
pub fn wrap_with_status<T, E>(value: result::Result<T, E>, status: i32) -> result::Result<T, Error>
        where E: StdError, {
    value.map_err(|e| Error::from_error(e, status))
}

/// Return a `Result` indicating successful execution, at which point, the process can exit with a
/// status code of `0`. Used as the terminal step in a computation, right before the program is
/// finished executing.
pub fn success() -> Result {
    Ok(())
}

pub fn exit_with(result: Result) -> ! {
    match result {
        Ok(_) => process::exit(0),
        Err(err) => match err {
            Error { message, status } => {
                println!("{}", message);
                process::exit(status)
            }
        },
    }
}
