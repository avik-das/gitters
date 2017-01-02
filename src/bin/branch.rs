extern crate gitters;

extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use gitters::cli;
use gitters::branch;

const USAGE: &'static str = "
branch - List branches

Usage:
  branch
  branch (-h | --help)

Options:
  -h --help  Show this screen.
";

#[derive(RustcDecodable)]
struct Args {}

fn list_branches() -> cli::Result {
    let current_branch = try!(cli::wrap_with_status(branch::current_branch(), 1));
    let all_branches = try!(cli::wrap_with_status(branch::all_branches(), 1));

    for branch in all_branches {
        if branch == current_branch {
            println!("\x1B[0;32m* {}\x1B[0m", branch);
        } else {
            println!("  {}", branch);
        }
    }

    cli::success()
}

fn main() {
    let _: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    cli::exit_with(list_branches())
}
