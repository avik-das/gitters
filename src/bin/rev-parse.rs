extern crate gitters;

extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use gitters::cli;
use gitters::objects;
use gitters::revisions;

const USAGE: &'static str = "
rev-parse - Pick out and massage parameters

Usage:
  rev-parse <revision>...
  rev-parse (-h | --help)

Options:
  -h --help  Show this screen.
";

#[derive(RustcDecodable)]
struct Args {
    arg_revision: Vec<String>,
}

fn parse_and_print_revisions(revs: Vec<String>) -> cli::Result {
    for rev in revs {
        let objects::Name(parsed) = try!(cli::wrap_with_status(revisions::resolve(&rev), 1));
        println!("{}", parsed);
    }

    cli::success()
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    cli::exit_with(parse_and_print_revisions(args.arg_revision))
}
