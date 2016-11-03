extern crate gitters;

extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
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

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let name = revisions::resolve(&args.arg_object).unwrap();
    let header = objects::read_header(&name).unwrap();

    if args.flag_t {
        let object_type = match header.object_type {
            objects::Type::Blob => "blob",
            objects::Type::Tree => "tree",
            objects::Type::Commit => "commit",
        };
        println!("{}", object_type);
    } else if args.flag_s {
        println!("{}", header.content_length);
    } else {
        // TODO
        println!("{:?}", header);
    }
}
