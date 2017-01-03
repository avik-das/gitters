extern crate gitters;

extern crate rustc_serialize;
extern crate docopt;
#[macro_use]
extern crate lazy_static;

use docopt::Docopt;
use gitters::cli;
use gitters::index;
use std::env;
use std::path::{Path, PathBuf};

const USAGE: &'static str = "
ls-files - Show information about files in the index

Usage:
  ls-files
  ls-files (-c | --cached)
  ls-files (-o | --others)
  ls-files (-h | --help)

Options:
  -h --help    Show this screen.
  -c --cached  Show cached files in the output (default).
  -o --others  Show other (i.e. untracked files) in the output
";

#[derive(RustcDecodable)]
struct Args {
    flag_o: bool,
}

fn path_display(path: &Path) -> Result<String, cli::Error> {
    lazy_static! {
        static ref CURRENT_DIR_PREFIX: PathBuf = env::current_dir().unwrap();
    }

    let relative_path =
        try!(cli::wrap_with_status(path.strip_prefix(CURRENT_DIR_PREFIX.as_path()), 2));
    Ok(relative_path.display().to_string())
}

fn list_cached_files() -> cli::Result {
    let index = try!(cli::wrap_with_status(index::Index::read(), 2));
    for entry in index.entries {
        println!("{}", try!(path_display(&entry.path)));
    }

    cli::success()
}

fn list_other_files() -> cli::Result {
    let files = try!(cli::wrap_with_status(index::untracked_files(), 2));

    let mut sorted_files = files
        .into_iter()
        .collect::<Vec<_>>();
    sorted_files.sort();

    for file in sorted_files {
        println!("{}", try!(path_display(&file)));
    }

    cli::success()
}

fn dispatch_for_args(args: &Args) -> cli::Result {
    if args.flag_o {
        list_other_files()
    } else {
        list_cached_files()
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    cli::exit_with(dispatch_for_args(&args))
}
