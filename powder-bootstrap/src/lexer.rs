#[repr(u64)]
#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug)]
pub struct Token<'a> {
	pub type_: TokenType,
	pub start: usize,
	pub end: usize,
	pub code: &'a String,
}

impl<'a> Token<'a> {
	pub fn text(&self) -> &'a str {
		self
			.code
			.get(self.start..self.end)
			.unwrap_or_else(|| panic!("Invalid token from {} to {}", self.start, self.end))
	}
}

#[derive(Debug)]
struct LexerState<'a> {
	pub code: &'a String,
	pub current_position: usize,
}

impl<'a> LexerState<'a> {
	pub fn is_end(&self) -> bool {
		self.current_position >= self.code.len()
	}

	pub fn current_char(&self) -> char {
		self
			.code
			.chars()
			.nth(self.current_position)
			.unwrap_or_else(|| panic!("Invalid lexer position {}", self.current_position))
	}

	pub fn skip_whitespace(&mut self) -> &Self {
		while self.current_char().is_whitespace() && !self.is_end() {
			self.advance();
		}
		self
	}

	/// Only works correctly if the first character was already checked to be non-numeric.
	pub fn next_word(&mut self) -> &'a str {
		let start = self.current_position;
		while (self.current_char().is_alphanumeric() || self.current_char() == '_') && !self.is_end()
		{
			self.advance();
		}
		&self.code[start..self.current_position]
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
pub fn lex(code: &String) -> Vec<Token> {
	let mut state = LexerState {
		code,
		current_position: 0,
	};

	let mut tokens = Vec::new();

	while !state.is_end() {
		state.skip_whitespace();
		println!("{}", state.current_char());

		let start = state.current_position;
		match state.current_char() {
			'/' => {
				// FIXME: Distinguish between / and //
				while state.current_char() != '\n' {
					if state.advance().is_none() {
						break;
					}
				}
				println!(
					"Found comment: '{}'",
					&state.code[start..state.current_position]
				);
				tokens.push(Token {
					start,
					end: state.current_position,
					code,
					type_: TokenType::Comment,
				});
			}
			'a'..='z' | 'A'..='Z' | '_' => {
				let word = state.next_word();
				if let Some(keyword_type) = TokenType::from_keyword(word) {
					println!("Found keyword '{}' of type {:?}", word, keyword_type);
					tokens.push(Token {
						start,
						end: state.current_position,
						code,
						type_: keyword_type,
					});
				} else {
					println!("Found identifier '{}'", word);
					tokens.push(Token {
						start,
						end: state.current_position,
						code,
						type_: TokenType::Identifer,
					});
				}
			}
			_ => panic!(),
		}
	}

	tokens
}
