extern crate gitters;

extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use gitters::cli;
use gitters::commits;
use gitters::objects;
use gitters::revisions;

const USAGE: &'static str = "
log

Usage:
  log <object>
  log (-h | --help)

Options:
  -h --help  Show this screen.
";

#[derive(RustcDecodable)]
struct Args {
    arg_object: String,
}

fn print_full_commit(commit: &commits::Commit) {
    let &commits::Commit {
        name: objects::Name(ref name),
        author: commits::CommitUser { name: ref author_name, date: ref author_date },
        ref message,
        ..
    } = commit;

    println!("\x1B[33mcommit {}\x1B[0m", name);
    println!("Author: {}", author_name);
    println!("Date:   {}", author_date.format("%a %b %-d %H:%M:%S %Y %z"));
    println!("");
    println!("    {}", str::replace(&message, "\n", "\n    "));
    println!("");
}

fn print_history(commit_rev: String) -> cli::Result {
    let resolved = try!(cli::wrap_with_status(revisions::resolve(&commit_rev), 1));

    let mut current_commit_rev = Some(resolved);
    while current_commit_rev.is_some() {
        let name = current_commit_rev.unwrap();
        let obj = try!(cli::wrap_with_status(objects::read_object(&name), 1));

        match obj {
            objects::Object::Commit(commit) => {
                // In the future, print in the format specified by the command line arguments.
                print_full_commit(&commit);
                current_commit_rev = commit.parent;
            },
            _ => {
                return Err(cli::Error {
                    message: format!("object {} is not a commit", name),
                    status: 2
                });
            }
        }
    }

    return cli::success();
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    cli::exit_with(print_history(args.arg_object))
}
