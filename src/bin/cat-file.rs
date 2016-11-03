extern crate gitters;

extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
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

fn bool_to_yn(b: bool) -> &'static str {
    if b { "Y" } else { "N" }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    println!("flags: t = {}, s = {}, e = {}, p = {}",
             bool_to_yn(args.flag_t), bool_to_yn(args.flag_s),
             bool_to_yn(args.flag_e), bool_to_yn(args.flag_p));

    // TODO Steps:
    // 1. Resolve revision -> error if invalid
    // 2. Read header if t, s or e. Read entire object if p -> error if unable to
    // 3. Print type, size or entire object as necessary.

    match revisions::resolve(&args.arg_object) {
        Ok(rev) => gitters::cat_file(rev),
        Err(err) => println!("{}", err),
    }
}
