extern crate gitters;

extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use gitters::cli;
use gitters::index;
use std::error::Error;

const USAGE: &'static str = "
ls-files - Show information about files in the index

Usage:
  ls-files
  ls-files (-h | --help)

Options:
  -h --help  Show this screen.
";

#[derive(RustcDecodable)]
struct Args {}

fn list_files_from_index() -> cli::Result {
    match index::Index::read() {
        Ok(index) => {
            for entry in index.entries {
                println!("{}", entry.path_name);
            }

            cli::success()
        },
        Err(ref err) => Err(cli::Error { message: err.description().to_string(), status: 2 }),
    }
}

fn main() {
    let _: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    cli::exit_with(list_files_from_index())
}
