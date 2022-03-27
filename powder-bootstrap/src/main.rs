//! Powder bootstrap compiler.
#![deny(clippy::all, clippy::pedantic, clippy::nursery, missing_docs)]

use log::Level;
use std::env::args;
use std::fs::read_to_string;

mod ast;
mod lexer;
mod parser;

fn main() {
	simple_logger::init_with_level(Level::Debug).expect("Couldn't initialize logger");

	let mut args = args();
	let filename = args.nth(1).expect("No file to parse given");
	let code = read_to_string(&filename).expect("I/O error while reading from file");
	let maybe_tokens = lexer::lex(&code);
	match maybe_tokens {
		Ok(tokens) => {
			println!(
				"{}",
				tokens
					.iter()
					.map(|t| format!("{}", t))
					.collect::<Vec<String>>()
					.join(" ")
			);
			match parser::parse(&tokens) {
				Ok(ast) => println!("{:#?}", ast),
				Err(why) => panic!("Error while parsing: {:?}", why),
			}
		}
		Err(why) => panic!("Error while lexing file {}: {}", filename, why),
	}
}
