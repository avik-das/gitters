//! A pared down implementation of `git config`, without all the bells and whistles. Supports
//! getting, setting and unsetting, from a hierarchy of files.

extern crate gitters;

extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use gitters::cli;
use gitters::config;

const USAGE: &'static str = "
config

Usage:
  config [options] <name> [<value>]
  config [options] (-l | --list)
  config (-h | --help)

Options:
  -h --help  Show this screen.
  --global   Use global config file.
  --local    Use repository config file.
  -l --list  List all.
";

#[derive(RustcDecodable)]
struct Args {
    flag_list: bool,
}

fn dispatch_for_args(args: &Args) -> cli::Result {
    if args.flag_list {
        let cfg = try!(cli::wrap_with_status(config::read_all(), 1));
        for (k, v) in cfg.all() {
            println!("{}={}", k, v);
        }
        cli::success()
    } else {
        Err(cli::Error { message: "Invalid options".to_string(), status: 2 })
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    cli::exit_with(dispatch_for_args(&args))
}
