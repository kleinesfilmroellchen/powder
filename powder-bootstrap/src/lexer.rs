use std::fmt::{Display, Formatter};

use log::debug;

#[repr(u64)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TokenType {
	// Comments
	Comment,
	// Keywords
	Function,
	Const,
	Var,
	// Type keywords
	N64,
	// Identifiers
	Identifer,
	// Literals
	IntegerLiteral,
	// Symbols
	OpenParenthesis,
	CloseParenthesis,
	OpenBrace,
	CloseBrace,
	Colon,
	Equals,
	Semicolon,
	Plus,
	Minus,
	Star,
	Slash,
}

impl TokenType {
	pub fn from_keyword(string: &str) -> Option<Self> {
		match string {
			"function" => Some(Self::Function),
			"const" => Some(Self::Const),
			"var" => Some(Self::Var),
			"n64" => Some(Self::N64),
			_ => None,
		}
	}

	pub const fn from_symbol(symbol: char) -> Option<Self> {
		match symbol {
			'(' => Some(Self::OpenParenthesis),
			')' => Some(Self::CloseParenthesis),
			'{' => Some(Self::OpenBrace),
			'}' => Some(Self::CloseBrace),
			':' => Some(Self::Colon),
			'=' => Some(Self::Equals),
			';' => Some(Self::Semicolon),
			'+' => Some(Self::Plus),
			'-' => Some(Self::Minus),
			'*' => Some(Self::Star),
			'/' => Some(Self::Slash),
			_ => None,
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Token<'a> {
	pub type_: TokenType,
	pub start: usize,
	pub end:   usize,
	pub code:  &'a str,
}

impl<'a> Token<'a> {
	pub fn text(&self) -> &'a str {
		self.code
			.get(self.start .. self.end)
			.unwrap_or_else(|| panic!("Invalid token from {} to {}", self.start, self.end))
	}

	pub fn expect(self, type_: TokenType) -> Result<Self, String> {
		debug!("Checking for type {:?}, but has {:?}", type_, self.type_);
		if self.type_ == type_ {
			Ok(self)
		} else {
			Err(format!("Expected '{:?}' token", type_))
		}
	}

	pub fn expect_any(self, types: &[TokenType]) -> Result<Self, String> {
		if types.contains(&self.type_) {
			Ok(self)
		} else {
			Err(format!("Expected any of '{:?}' token", types))
		}
	}
}

impl Display for Token<'_> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{:?}({})", self.type_, self.text())
	}
}

#[derive(Debug)]
struct LexerState<'a> {
	pub code:             &'a str,
	pub current_position: usize,
}

impl<'a> LexerState<'a> {
	pub const fn is_end(&self) -> bool {
		self.current_position >= self.code.len()
	}

	pub fn current_char(&self) -> char {
		self.code
			.chars()
			.nth(self.current_position)
			.unwrap_or_else(|| panic!("Invalid lexer position {}", self.current_position))
	}

	pub fn next_char(&self, lookahead: usize) -> Option<char> {
		self.code.chars().nth(self.current_position + lookahead)
	}

	pub fn skip_whitespace(&mut self) -> &Self {
		while !self.is_end() && self.current_char().is_whitespace() {
			self.advance();
		}
		self
	}

	/// Read the longest continuous sequence of characters, starting from the current position, for which the predicate
	/// returns true.
	pub fn next_of_kind(&mut self, predicate: fn(&char) -> bool) -> &'a str {
		let start = self.current_position;
		while !self.is_end() && predicate(&self.current_char()) {
			self.advance();
		}
		&self.code[start .. self.current_position]
	}

	/// Only works correctly if the first character was already checked to be non-numeric.
	pub fn next_word(&mut self) -> &'a str {
		self.next_of_kind(|character| character.is_alphanumeric() || character == &'_')
	}

	pub fn next_number_sequence(&mut self) -> &'a str {
		self.next_of_kind(char::is_ascii_digit)
	}

	pub fn advance(&mut self) -> Option<&Self> {
		self.current_position += 1;
		if self.is_end() {
			None
		} else {
			Some(self)
		}
	}
}
#[allow(clippy::ptr_arg)]
pub fn lex(code: &str) -> Result<Vec<Token>, String> {
	let mut state = LexerState { code, current_position: 0 };

	let mut tokens = Vec::new();

	while !state.is_end() {
		state.skip_whitespace();
		if state.is_end() {
			break;
		}

		let start = state.current_position;
		match state.current_char() {
			'/' => {
				if let Some(maybe_slash) = state.next_char(1) && maybe_slash == '/' {
					while state.current_char() != '\n' {
						if state.advance().is_none() {
							break;
						}
					}
					debug!(
						"Found comment: '{}'",
						&state.code[start..state.current_position]
					);
					tokens.push(Token {
						start,
						end: state.current_position,
						code,
						type_: TokenType::Comment,
					});
				} else {
					state.advance();
					tokens.push(Token {
						start,
						end: start+1,
						code,
						type_: TokenType::Slash,
					});
				}
			}
			'a'..='z' | 'A'..='Z' | '_' => {
				let word = state.next_word();
				if let Some(keyword_type) = TokenType::from_keyword(word) {
					debug!("Found keyword '{}' of type {:?}", word, keyword_type);
					tokens.push(Token {
						start,
						end: state.current_position,
						code,
						type_: keyword_type,
					});
				} else {
					debug!("Found identifier '{}'", word);
					tokens.push(Token {
						start,
						end: state.current_position,
						code,
						type_: TokenType::Identifer,
					});
				}
			}
			'0'..='9' => {
				let number = state.next_number_sequence();
				// FIXME: Parse base prefixes (0x, 0o, 0b)
				debug!("Found number '{}'", number);
				tokens.push(Token {
					start,
					end: state.current_position,
					code,
					type_: TokenType::IntegerLiteral,
				});
			}
			_ => {
				// single-character symbols
				let symbol_char = state.current_char();
				state.advance();
				if let Some(symbol_type) = TokenType::from_symbol(symbol_char) {
					debug!("Found symbol '{}' of type {:?}", symbol_char, symbol_type);
					tokens.push(Token {
						start,
						end: state.current_position,
						code,
						type_: symbol_type,
					});
				} else {
					return Err(format!("Unexpected token '{}'", symbol_char));
				}
			}
		}
	}

	Ok(tokens)
}
