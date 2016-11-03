#[macro_use]
extern crate lazy_static;
extern crate regex;

pub mod objects;
pub mod revisions;

pub fn cat_file(src: &objects::ObjectName) {
    println!("TODO: implement cat-file");
    println!("Show contents of {}", src);
}
