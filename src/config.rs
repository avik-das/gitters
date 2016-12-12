//! Provides functionality for building up a model of the configuration files used by git, as well
//! as editing them.

use std::{env, fmt, io, str};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fs::File;
use std::iter::Peekable;
use std::io::Read;
use std::str::Chars;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidFile(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IOError(ref err) => write!(f, "IO error: {}", err),
            Error::InvalidFile(ref description) => write!(f, "invalid file: {}", description),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IOError(ref err) => err.description(),
            Error::InvalidFile(ref description) => description,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::IOError(ref err) => Some(err),
            ref err => Some(err)
        }
    }
}

struct Parser<'a> {
    map: HashMap<String, String>,
    chars: Peekable<Chars<'a>>,
    current_section_names: Vec<String>,
}

impl <'a> Parser<'a> {
    fn new(contents: &'a String) -> Parser<'a> {
        Parser {
            map: HashMap::new(),
            chars: contents.chars().peekable(),
            current_section_names: Vec::new(),
        }
    }

    fn parse(&mut self) -> Result<(), Error> {
        loop {
            match self.chars.peek() {
                Some(&' ') | Some(&'\t') => { self.chars.next(); },
                Some(&'#') => try!(self.parse_comment()),
                Some(&'[') => try!(self.parse_section()),
                Some(_) => { self.chars.next(); },
                None => { self.chars.next(); break; },
            }
        }

        Ok(())
    }

    fn parse_comment(&mut self) -> Result<(), Error> {
        loop {
            match self.chars.next() {
                Some('\n') | None => break,
                _ => continue,
            }
        }

        Ok(())
    }

    fn parse_section(&mut self) -> Result<(), Error> {
        let mut section_name = String::new();

        self.chars.next();
        loop {
            match self.chars.peek() {
                Some(&']') | None => {
                    self.chars.next();
                    self.current_section_names.push(section_name);
                    try!(self.parse_variables());
                    self.current_section_names.pop();
                    break;
                },
                Some(&chr) => {
                    section_name.push(chr);
                    self.chars.next();
                }
            }
        }

        Ok(())
    }

    fn parse_variables(&mut self) -> Result<(), Error> {
        loop {
            match self.chars.peek() {
                Some(&'#') => try!(self.parse_comment()),
                Some(&' ') | Some(&'\t') => { self.chars.next(); },
                Some(&'[') | None => break,
                _ => try!(self.parse_single_variable()),
            }
        }

        Ok(())
    }

    fn parse_single_variable(&mut self) -> Result<(), Error> {
        let mut key_name = String::new();
        loop {
            match self.chars.peek() {
                Some(&'#') => { try!(self.parse_comment()); break; },
                Some(&' ') | Some(&'\t') => { self.chars.next(); },
                Some(&'\n') => { self.chars.next(); break; },
                Some(&'=') => {
                    self.chars.next();
                    try!(self.parse_key_value_pair(key_name.clone()));
                    key_name.clear();
                    break;
                }
                Some(&'[') | None => break,
                Some(&chr) => {
                    self.chars.next();
                    key_name.push(chr);
                },
            }
        }

        if !key_name.is_empty() {
            let full_key_name = self.variable_name_for_current_section(&key_name);
            self.map.insert(full_key_name, "true".to_string());
        }
        Ok(())
    }

    fn parse_key_value_pair(&mut self, key_name: String) -> Result<(), Error> {
        let mut value = String::new();

        loop {
            match self.chars.peek() {
                Some(&'#') => { try!(self.parse_comment()); break; },
                Some(&'\n') => { self.chars.next(); break; },
                Some(&'[') | None => break,
                Some(&chr) => {
                    self.chars.next();
                    value.push(chr);
                },
            }
        }

        let full_key_name = self.variable_name_for_current_section(&key_name);
        self.map.insert(full_key_name, value.trim().to_string());
        Ok(())
    }

    fn variable_name_for_current_section(&self, key_name: &String) -> String {
        let mut name = String::new();
        for section in self.current_section_names.iter() {
            name.push_str(&section);
            name.push('.');
        }

        name.push_str(key_name);
        name
    }
}

/// The fundamental data structure representing the configuration for this process. Instead of
/// having specific fields for each configuration item, this structure exposes a map-like interface
/// indexed by strings.
pub struct Config {
    map: HashMap<String, String>,
}

impl Config {
    fn new() -> Config {
        Config { map: HashMap::new() }
    }

    fn add_from_file(&mut self, filename: String) -> Result<&Config, Error> {
        println!("reading from {}", filename);
        let mut file = try!(File::open(filename).map_err(|e| Error::IOError(e)));
        let mut contents = String::new();
        try!(file.read_to_string(&mut contents).map_err(|e| Error::IOError(e)));
        self.add_from_string(contents.to_string())
    }

    fn add_from_string(&mut self, contents: String) -> Result<&Config, Error> {
        let mut parser = Parser::new(&contents);
        try!(parser.parse());

        for (k, v) in parser.map.drain() {
            self.map.insert(k.to_string(), v.to_string());
        }

        Ok(self)
    }

    pub fn all(&self) -> Vec<(String, String)> {
        let mut list: Vec<(String, String)> = self.map
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        list.sort();
        list
    }
}

pub fn read_all() -> Result<Config, Error> {
    let mut config = Config::new();

    let home_gitconfig = env::home_dir()
        .and_then(|mut path| {
            path.push(".gitconfig");
            path.to_str().map(|s| s.to_string())
        });
    match home_gitconfig {
        Some(path) => { try!(config.add_from_file(path)); },
        None => {}
    }

    let repo_gitconfig = env::current_dir()
        .ok()
        .and_then(|mut path| {
            path.push(".git/config");
            path.to_str().map(|s| s.to_string())
        });
    match repo_gitconfig {
        Some(path) => { try!(config.add_from_file(path)); },
        None => {}
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config_syntax() {
        let contents = r"
# This is a comment
[simple]
key0 = val0
key1=val1
key2 = val2 with spaces

[Complicated-123] key3 = val3
key4
key-5 # here's a comment
key6 = val6 # and another comment

# TODO: multiline values
# TODO: subsections
";

        let mut config = Config::new();
        config.add_from_string(contents.to_string()).unwrap();
        assert_eq!(config.all(), vec![
                   ("Complicated-123.key-5", "true"),
                   ("Complicated-123.key3", "val3"),
                   ("Complicated-123.key4", "true"),
                   ("Complicated-123.key6", "val6"),
                   ("simple.key0", "val0"),
                   ("simple.key1", "val1"),
                   ("simple.key2", "val2 with spaces"),
                   ]
                   .iter()
                   .map(|s| (s.0.to_string(), s.1.to_string()))
                   .collect::<Vec<_> >());
    }
}
