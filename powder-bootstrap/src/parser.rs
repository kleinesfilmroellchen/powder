use crate::ast::{
	Ast, Block, Definition, Expression, File, Function, Statement, Type, VariableKind,
};
use crate::lexer::{Token, TokenType};
use log::debug;
use std::iter::Peekable;

pub fn parse(tokens: &[Token]) -> Result<Box<dyn Ast>, String> {
	let mut definitions = Vec::<Definition>::new();

	let mut token_iterator = tokens.iter().peekable();
	while let Some(next_token) = token_iterator.next() {
		if next_token.type_ == TokenType::Comment {
			continue;
		}

		// TODO: Deal with comments everywhere.
		definitions.push(parse_definition(*next_token, &mut token_iterator)?);
	}

	Ok(Box::new(File { definitions }))
}

fn parse_definition(
	next_token: Token,
	token_iterator: &mut Peekable<std::slice::Iter<Token>>,
) -> Result<Definition, String> {
	if next_token.expect(TokenType::Function).is_ok() {
		let function_name = token_iterator
			.next()
			.ok_or("Expected function name")?
			.expect(TokenType::Identifer)?;
		token_iterator
			.next()
			.ok_or("Expected '('")?
			.expect(TokenType::OpenParenthesis)?;
		// TODO: Parse arguments
		token_iterator
			.next()
			.ok_or("Expected ')'")?
			.expect(TokenType::CloseParenthesis)?;

		let function_body = parse_block(
			*token_iterator.next().ok_or("Expected '{'")?,
			token_iterator,
		)?;

		token_iterator
			.next()
			.ok_or("Expected '}'")?
			.expect(TokenType::CloseBrace)?;

		Ok(Definition::Function(Function {
			name: function_name.text().to_string(),
			body: function_body,
		}))
	} else {
		let first_function_token = token_iterator
			.next()
			.ok_or("Expected variable declaration")?
			.expect(TokenType::Var)
			.or_else(|_| next_token.expect(TokenType::Const))
			.map_err(|_| String::from("Expected 'const' or 'var'"))?;
		Ok(Definition::Statement(parse_statement(
			first_function_token,
			token_iterator,
		)?))
	}
}

fn parse_block(
	next_token: Token,
	token_iterator: &mut Peekable<std::slice::Iter<Token>>,
) -> Result<Block, String> {
	next_token.expect(TokenType::OpenBrace)?;

	let mut statements = Vec::<Statement>::new();

	let mut maybe_peeked_next = token_iterator.peek();
	while maybe_peeked_next.map_or(false, |peeked_next| {
		peeked_next.type_ != TokenType::CloseBrace
	}) {
		statements.push(parse_statement(
			*token_iterator.next().ok_or("Expected something")?,
			token_iterator,
		)?);
		maybe_peeked_next = token_iterator.peek();
	}

	// TODO: Parse last expression
	Ok(Block {
		statements,
		value: None,
	})
}

fn parse_statement(
	next_token: Token,
	token_iterator: &mut Peekable<std::slice::Iter<Token>>,
) -> Result<Statement, String> {
	// TODO: Parse other statement types
	let kind = next_token
		.expect(TokenType::Var)
		.or_else(|_| next_token.expect(TokenType::Const))
		.map_err(|_| String::from("Expected 'const' or 'var'"))?;

	let identifier = token_iterator
		.next()
		.ok_or("Expected identifier")?
		.expect(TokenType::Identifer)?;
	token_iterator
		.next()
		.ok_or("Expected ':'")?
		.expect(TokenType::Colon)?;
	// TODO: Allow any type
	let _type = token_iterator
		.next()
		.ok_or("Expected type")?
		.expect(TokenType::N64)?;

	// Get the initializer if it exists
	let initializer = if let Ok(_equals) = token_iterator
		.peek()
		.ok_or("Expected ':' or ';'")?
		.expect(TokenType::Equals)
	{
		token_iterator.next();
		let expression = parse_expression(
			*token_iterator.next().ok_or("Expected expression")?,
			token_iterator,
		)?;
		token_iterator
			.next()
			.ok_or("Expected ';'")?
			.expect(TokenType::Semicolon)?;
		Some(expression)
	} else {
		token_iterator
			.next()
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

fn parse_expression(
	next_token: Token,
	token_iterator: &mut Peekable<std::slice::Iter<Token>>,
) -> Result<Expression, String> {
	// TODO: Parse other expressions
	if next_token.type_ == TokenType::Plus {
		debug!("Unary plus found");
		Ok(Expression::UnaryPlus(Box::new(parse_expression(
			*token_iterator.next().ok_or("Expected expression")?,
			token_iterator,
		)?)))
	} else {
		let number_literal = next_token.expect(TokenType::IntegerLiteral)?;
		// TODO: Use our own integer parser
		let value: u128 = number_literal
			.text()
			.parse()
			.map_err(|err| format!("Invalid number literal: {}", err))?;
		Ok(Expression::NaturalLiteral(value))
	}
}
