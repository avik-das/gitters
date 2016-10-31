extern crate gitters;

// TODO: start using the docopt library
use std::env;

fn main() {
    gitters::cat_file(&env::args().last().unwrap());
}
