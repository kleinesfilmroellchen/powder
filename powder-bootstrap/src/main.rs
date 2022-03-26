//! Powder bootstrap compiler.
#![deny(clippy::all, clippy::pedantic, clippy::nursery, missing_docs)]

use std::env::args;
use std::fs::read_to_string;

mod lexer;

fn main() {
	let mut args = args();
	let filename = args.nth(1).expect("No file to parse given");
	let code = read_to_string(filename).expect("I/O error while reading from file");
	let tokens = lexer::lex(&code);
	println!("{:?}", tokens);
}
