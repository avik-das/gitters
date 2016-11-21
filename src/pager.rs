//! Sets up the process so that everything printed to STDOUT goes to a pager, if configured. This
//! module exposes one main entry point: `setup`. The setup function does all the work such that
//! after the call, content can be printed to STDOUT and it will automatically be displayed in the
//! pager.
//!
//! TODO: need to implement this part
//! The pager is chosen based on the following, in the specified order:
//!
//! - `$GIT_PAGER`
//! - `core.pager`
//! - `$PAGER`
//! - compile-time default
//!
//! This code is mostly copied, but simplied and adapted, from the pager-rs project at
//! https://gitlab.com/imp/pager-rs. That code is under the Apache 2 and MIT licenses.

extern crate errno;
extern crate libc;

use std::error::Error as StdError;
use std::ffi::{CString, OsString};
use std::fmt;
use std::os::unix::ffi::OsStringExt;
use std::ptr;

#[derive(Debug)]
pub enum Error {
    SetupError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::SetupError(ref description) => write!(f, "setup error: {}", description),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::SetupError(ref description) => description,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}


macro_rules! check_libc_call {
    ($success: expr, $msg: expr) => {
        if !$success {
            return Err(Error::SetupError($msg.to_string()));
        }
    }
}

// TODO: accept config object
pub fn setup() -> Result<(), Error> {
    // TODO: find correct pager command. This should also involve checking if we're outputting to a
    // tty, and setting up environment variables like in
    // https://github.com/git/git/blob/398dd4bd039680ba98497fbedffa415a43583c16/pager.c#L83-L93
    let cmd = "less -R";

    let mut pipe_fds = [0; 2];
    unsafe { libc::pipe(pipe_fds.as_mut_ptr()); } // TODO: error checking
    let (pager_stdin, main_stdout) = (pipe_fds[0], pipe_fds[1]);

    match unsafe { libc::fork() } {
        -1 => {
            // Fork failed. Clean up.
            unsafe {
                // Don't bother with error checking. The setup failed anyway.
                libc::close(pager_stdin);
                libc::close(main_stdout);
            }

            Err(Error::SetupError("unable to fork".to_string()))
        },
        0 => {
            // We are in the child process. This will continue running the current program, but
            // with STDOUT pointing to the output end of the created pipe. Close the input end of
            // the pipe because only the parent process will be reading from the pipe.
            unsafe {
                check_libc_call!(
                    libc::dup2(main_stdout, libc::STDOUT_FILENO) > -1,
                    "unable to reroute STDOUT");
                check_libc_call!(libc::close(pager_stdin) == 0, "unable to close STDIN");
            }

            Ok(())
        },
        _ => {
            // We are in the parent process. Replace this process with the pager, but with the
            // STDIN pointing to the input end of the created pipe. Close the output end of the
            // pipe because the child process is the one that will be writing to the pipe.
            unsafe {
                check_libc_call!(
                    libc::dup2(pager_stdin, libc::STDIN_FILENO) > -1,
                    "unable to reroute STDIN");
                check_libc_call!(libc::close(main_stdout) == 0, "unable to close STDOUT");

                let cstrings = cmd
                    .split_whitespace()
                    .map(|s| {
                        let bytes = OsString::from(s).into_vec();
                        CString::from_vec_unchecked(bytes)
                    })
                    .collect::<Vec<_>>();

                let mut args = cstrings
                    .iter()
                    .map(|c| c.as_ptr())
                    .collect::<Vec<_>>();
                args.push(ptr::null());

                errno::set_errno(errno::Errno(0));
                libc::execvp(args[0], args.as_ptr());
            }

            Ok(())
        }
    }
}
