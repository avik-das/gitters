extern crate gitters;

extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use gitters::objects;
use gitters::revisions;
use std::process;

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

fn show_type(name: &objects::Name) -> Result<(), i32> {
    let header = try!(objects::read_header(&name).map_err(|_| 1));

    let object_type = match header.object_type {
        objects::Type::Blob => "blob",
        objects::Type::Tree => "tree",
        objects::Type::Commit => "commit",
    };
    println!("{}", object_type);

    Ok(())
}

fn show_size(name: &objects::Name) -> Result<(), i32> {
    let header = try!(objects::read_header(&name).map_err(|_| 1));
    println!("{}", header.content_length);

    Ok(())
}

fn check_validity(name: &objects::Name) -> Result<(), i32> {
    try!(objects::read_header(&name).map_err(|_| 1));
    Ok(())
}

fn show_contents(name: &objects::Name) -> Result<(), i32> {
    let header = try!(objects::read_header(&name).map_err(|_| 1));
    // TODO

    Ok(())
}

fn dispatch_for_args(args: &Args) -> Result<(), i32> {
    let name = revisions::resolve(&args.arg_object).unwrap();

    if args.flag_t {
        show_type(&name)
    } else if args.flag_s {
        show_size(&name)
    } else if args.flag_e {
        check_validity(&name)
    } else if args.flag_p {
        show_contents(&name)
    } else {
        Err(2)
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    match dispatch_for_args(&args) {
        Ok(_) => process::exit(0),
        Err(status) => process::exit(status),
    }
}
