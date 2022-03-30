use crate::ast::{
	Ast, Block, Definition, Expression, File, Function, Statement, Type, VariableKind,
};
use crate::lexer::{Token, TokenType};

// TODO: Include the code string with this struct. That makes it self-referential, though, so we would need ouboros (?).
#[derive(Debug)]
pub struct TokenStream<'a> {
	tokens: Vec<Token<'a>>,
	index: usize,
}

impl<'a> TokenStream<'a> {
	pub fn new(tokens: Vec<Token<'a>>) -> Self {
		Self {
			tokens: tokens
				.into_iter()
				.filter(|token| token.type_ != TokenType::Comment)
				.collect(),
			index: 0,
		}
	}

	pub fn len(&self) -> usize {
		self.tokens.len()
	}

	pub fn is_end(&self) -> bool {
		self.index >= self.len()
	}

	pub fn next(&mut self, error_message: &str) -> Result<Token<'a>, String> {
		if self.is_end() {
			Err(error_message.to_owned())
		} else {
			self.index += 1;
			Ok(self.tokens[self.index - 1])
		}
	}

	pub fn lookahead(&mut self, amount: usize, error_message: &str) -> Result<&[Token<'a>], String> {
		self
			.tokens
			.get(self.index..self.index + amount)
			.ok_or_else(|| error_message.to_owned())
	}
}

pub fn parse(tokens: Vec<Token>) -> Result<Box<dyn Ast>, String> {
	let mut definitions = Vec::<Definition>::new();

	let mut tokens = TokenStream::new(tokens);

	while !tokens.is_end() {
		definitions.push(parse_definition(&mut tokens)?);
	}

	Ok(Box::new(File { definitions }))
}

fn parse_definition(token_iterator: &mut TokenStream) -> Result<Definition, String> {
	let first_token = token_iterator.lookahead(1, "Expected definition")?[0];
	if first_token.expect(TokenType::Function).is_ok() {
		token_iterator.next("")?;
		let function_name = token_iterator
			.next("Expected function name")?
			.expect(TokenType::Identifer)?;
		token_iterator
			.next("Expected '('")?
			.expect(TokenType::OpenParenthesis)?;
		// TODO: Parse arguments
		token_iterator
			.next("Expected ')'")?
			.expect(TokenType::CloseParenthesis)?;

		let function_body = parse_block(token_iterator)?;

		token_iterator
			.next("Expected '}'")?
			.expect(TokenType::CloseBrace)?;

		Ok(Definition::Function(Function {
			name: function_name.text().to_string(),
			body: function_body,
		}))
	} else {
		Ok(Definition::Statement(parse_statement(token_iterator)?))
	}
}

fn parse_block(token_iterator: &mut TokenStream) -> Result<Block, String> {
	token_iterator
		.next("Expected '{'")?
		.expect(TokenType::OpenBrace)?;

	let mut statements = Vec::<Statement>::new();

	let mut maybe_peeked_next = token_iterator
		.lookahead(1, "Expected statement")
		.map(|peeked_next| peeked_next[0]);
	while maybe_peeked_next.map_or(false, |peeked_next| {
		peeked_next.type_ != TokenType::CloseBrace
	}) {
		statements.push(parse_statement(token_iterator)?);
		maybe_peeked_next = token_iterator
			.lookahead(1, "Expected statement")
			.map(|peeked_next| peeked_next[0]);
	}

	// TODO: Parse last expression
	Ok(Block {
		statements,
		value: None,
	})
}

fn parse_statement(token_iterator: &mut TokenStream) -> Result<Statement, String> {
	// TODO: Parse other statement types
	let kind = token_iterator
		.next("Expected statement")?
		.expect_any(&[TokenType::Var, TokenType::Const])?;

	let identifier = token_iterator
		.next("Expected identifier")?
		.expect(TokenType::Identifer)?;
	token_iterator
		.next("Expected ':'")?
		.expect(TokenType::Colon)?;
	// TODO: Allow any type
	let _type = token_iterator
		.next("Expected type")?
		.expect(TokenType::N64)?;

	// Get the initializer if it exists
	let initializer = if let Ok(_equals) =
		token_iterator.lookahead(1, "Expected '=' or ';'")?[0].expect(TokenType::Equals)
	{
		token_iterator.next("")?;
		let expression = parse_expression(token_iterator)?;
		token_iterator
			.next("Expected ';'")?
			.expect(TokenType::Semicolon)?;
		Some(expression)
	} else {
		token_iterator
			.next("")
			.unwrap()
			.expect(TokenType::Semicolon)?;
		None
	};

	Ok(Statement::VariableDeclaration {
		kind: if kind.type_ == TokenType::Const {
			VariableKind::Immutable
		} else {
			VariableKind::Mutable
		},
		name: identifier.text().to_string(),
		type_: Type::N64,
		initial_value: initializer,
	})
}

fn parse_expression(token_iterator: &mut TokenStream) -> Result<Expression, String> {
	// TODO: Parse other expressions
	if let Ok(_equals) =
		token_iterator.lookahead(1, "Expected expression")?[0].expect(TokenType::Plus)
	{
		token_iterator.next("")?;
		Ok(Expression::UnaryPlus(Box::new(parse_expression(
			token_iterator,
		)?)))
	} else {
		let number_literal = token_iterator
			.next("Expected literal")?
			.expect(TokenType::IntegerLiteral)?;
		// TODO: Use our own integer parser
		let value: u128 = number_literal
			.text()
			.parse()
			.map_err(|err| format!("Invalid number literal: {}", err))?;
		Ok(Expression::NaturalLiteral(value))
	}
}
