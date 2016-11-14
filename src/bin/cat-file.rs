extern crate gitters;

extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use gitters::cli;
use gitters::commits;
use gitters::objects;
use gitters::revisions;

const USAGE: &'static str = "
cat-file

Usage:
  cat-file -t <object>
  cat-file -s <object>
  cat-file -e <object>
  cat-file -p <object>
  cat-file (-h | --help)

Options:
  -h --help  Show this screen.
  -t         Instead of the content, show the object type identified by <object>.
  -s         Instead of the content, show the object size identified by <object>.
  -e         Surpress all output; instead exit with zero status if <object> exists and is a valid
             object.
  -p         Pretty-print the contents of <object> based on its type.
";

#[derive(RustcDecodable)]
struct Args {
    flag_t: bool,
    flag_s: bool,
    flag_e: bool,
    flag_p: bool,
    arg_object: String
}

fn show_type(name: &objects::Name) -> cli::Result {
    let header = try!(cli::wrap_with_status(objects::read_header(&name), 1));

    let object_type = match header.object_type {
        objects::Type::Blob => "blob",
        objects::Type::Tree => "tree",
        objects::Type::Commit => "commit",
    };
    println!("{}", object_type);

    cli::success()
}

fn show_size(name: &objects::Name) -> cli::Result {
    let header = try!(cli::wrap_with_status(objects::read_header(&name), 1));
    println!("{}", header.content_length);

    cli::success()
}

fn check_validity(name: &objects::Name) -> cli::Result {
    try!(cli::wrap_with_status(objects::read_header(&name), 1));
    cli::success()
}

fn show_contents(name: &objects::Name) -> cli::Result {
    let obj = try!(cli::wrap_with_status(objects::read_object(&name), 1));
    match obj {
        objects::Object::Commit(commit) => {
            let objects::Name(name) = commit.name;
            println!("commit {}", name);

            let objects::Name(tree) = commit.tree;
            println!("tree     : {}", tree);

            if commit.parent.is_some() {
                let objects::Name(parent) = commit.parent.unwrap();
                println!("parent   : {}", parent);
            }

            let commits::CommitUser { name, date } = commit.author;
            println!("author   : {} at {}", name, date);

            let commits::CommitUser { name, date } = commit.committer;
            println!("committer: {} at {}", name, date);

            println!("");
            println!("{}", commit.message);
        },
        _ => { /* Not handled yet */ }
    }

    cli::success()
}

fn dispatch_for_args(args: &Args) -> cli::Result {
    let name = try!(cli::wrap_with_status(revisions::resolve(&args.arg_object), 1));

    if args.flag_t {
        show_type(&name)
    } else if args.flag_s {
        show_size(&name)
    } else if args.flag_e {
        check_validity(&name)
    } else if args.flag_p {
        show_contents(&name)
    } else {
        Err(cli::Error { message: "No flags specified".to_string(), status: 2 })
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    cli::exit_with(dispatch_for_args(&args))
}
