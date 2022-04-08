use log::debug;

use crate::ast::{
	Ast, BinaryOperator, Block, Definition, Expression, File, Function, Statement, Type, UnaryOperator, VariableKind,
};
use crate::lexer::{Token, TokenType};

// TODO: Include the code string with this struct. That makes it self-referential, though, so we would need ouboros (?).
#[derive(Debug, Clone, Copy)]
pub struct TokenStream<'a> {
	tokens: &'a [Token<'a>],
	index:  usize,
}

impl<'a> TokenStream<'a> {
	pub const fn new(tokens: &'a [Token<'a>]) -> Self {
		Self { tokens, index: 0 }
	}

	pub fn strip_comments(tokens: Vec<Token<'a>>) -> Vec<Token<'a>> {
		tokens.into_iter().filter(|token| token.type_ != TokenType::Comment).collect()
	}

	pub const fn len(&self) -> usize {
		self.tokens.len()
	}

	pub const fn is_end(&self) -> bool {
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

	pub fn lookahead(&self, amount: usize, error_message: &str) -> Result<&[Token<'a>], String> {
		self.tokens.get(self.index .. self.index + amount).ok_or_else(|| error_message.to_owned())
	}

	#[allow(clippy::cast_possible_wrap)]
	pub fn backtrack(&mut self, amount: usize, error_message: &str) -> Result<(), String> {
		if (self.index as isize - amount as isize) < 0 {
			Err(error_message.to_owned())
		} else {
			self.index -= amount;
			Ok(())
		}
	}

	pub fn make_substream(&self) -> Self {
		Self { tokens: &self.tokens[self.index ..], index: 0 }
	}

	pub fn limit_to_first(&mut self, type_: TokenType) -> &Self {
		let mut current_index = self.index;
		while let Some(next_token) = self.tokens.get(current_index ..= current_index) {
			if next_token[0].type_ == type_ {
				break;
			}
			current_index += 1;
		}
		self.tokens = &self.tokens[.. current_index];

		self
	}

	pub fn advance_past_other(&mut self, other_stream: &Self) -> Result<&Self, String> {
		let other_token = other_stream.lookahead(1, "Expected any token")?[0];
		while let Ok(next_token) = self.next("") {
			if next_token == other_token {
				break;
			}
		}

		Ok(self)
	}
}

pub fn parse(tokens: Vec<Token>) -> Result<Box<dyn Ast>, String> {
	let mut definitions = Vec::<Definition>::new();

	let stripped_tokens = TokenStream::strip_comments(tokens);
	let mut token_stream = TokenStream::new(&stripped_tokens);

	while !token_stream.is_end() {
		definitions.push(parse_definition(&mut token_stream)?);
	}

	Ok(Box::new(File { definitions }))
}

fn parse_definition(token_iterator: &mut TokenStream) -> Result<Definition, String> {
	let first_token = token_iterator.lookahead(1, "Expected definition")?[0];
	if first_token.expect(TokenType::Function).is_ok() {
		token_iterator.next("")?;
		let function_name = token_iterator.next("Expected function name")?.expect(TokenType::Identifer)?;
		token_iterator.next("Expected '('")?.expect(TokenType::OpenParenthesis)?;
		// TODO: Parse arguments
		token_iterator.next("Expected ')'")?.expect(TokenType::CloseParenthesis)?;

		let function_body = parse_block(token_iterator)?;

		token_iterator.next("Expected '}'")?.expect(TokenType::CloseBrace)?;

		Ok(Definition::Function(Function { name: function_name.text().to_string(), body: function_body }))
	} else {
		Ok(Definition::Statement(parse_statement(token_iterator)?))
	}
}

fn parse_block(token_iterator: &mut TokenStream) -> Result<Block, String> {
	token_iterator.next("Expected '{'")?.expect(TokenType::OpenBrace)?;

	let mut statements = Vec::<Statement>::new();

	let mut maybe_peeked_next = token_iterator.lookahead(1, "Expected statement").map(|peeked_next| peeked_next[0]);
	while maybe_peeked_next.map_or(false, |peeked_next| peeked_next.type_ != TokenType::CloseBrace) {
		statements.push(parse_statement(token_iterator)?);
		maybe_peeked_next = token_iterator.lookahead(1, "Expected statement").map(|peeked_next| peeked_next[0]);
	}

	// TODO: Parse last expression
	Ok(Block { statements, value: None })
}

fn parse_statement(token_iterator: &mut TokenStream) -> Result<Statement, String> {
	// TODO: Parse other statement types
	let kind = token_iterator.next("Expected statement")?.expect_any(&[TokenType::Var, TokenType::Const])?;

	let identifier = token_iterator.next("Expected identifier")?.expect(TokenType::Identifer)?;
	token_iterator.next("Expected ':'")?.expect(TokenType::Colon)?;
	// TODO: Allow any type
	let _type = token_iterator.next("Expected type")?.expect(TokenType::N64)?;

	// Get the initializer if it exists
	let initializer =
		if let Ok(_equals) = token_iterator.lookahead(1, "Expected '=' or ';'")?[0].expect(TokenType::Equals) {
			token_iterator.next("")?;
			let mut expression_stream = token_iterator.make_substream();
			expression_stream.limit_to_first(TokenType::Semicolon);
			// debug!("{:#?}", expression_stream);
			let expression = parse_expression(&mut expression_stream)?;
			expression_stream.backtrack(1, ":yaksplode:")?;
			token_iterator.advance_past_other(&expression_stream)?;
			token_iterator.next("Expected ';'")?.expect(TokenType::Semicolon)?;
			Some(expression)
		} else {
			token_iterator.next("").unwrap().expect(TokenType::Semicolon)?;
			None
		};

	Ok(Statement::VariableDeclaration {
		kind:          if kind.type_ == TokenType::Const { VariableKind::Immutable } else { VariableKind::Mutable },
		name:          identifier.text().to_string(),
		type_:         Type::N64,
		initial_value: initializer,
	})
}

fn parse_expression(token_iterator: &mut TokenStream) -> Result<Expression, String> {
	parse_binary_operation(token_iterator, parse_term, &[TokenType::Plus, TokenType::Minus])
}

fn parse_binary_operation(
	token_iterator: &mut TokenStream,
	sub_operation_parser: fn(&mut TokenStream) -> Result<Expression, String>,
	operators: &[TokenType],
) -> Result<Expression, String> {
	let mut lhs = sub_operation_parser(token_iterator)?;

	let mut maybe_operator =
		token_iterator.lookahead(1, &format!("Expected any of {:?}", operators)).map(|tokens| tokens[0]);
	while let Ok(operator) = maybe_operator {
		if !operators.contains(&operator.type_) {
			break;
		}
		token_iterator.next("")?;
		let rhs = sub_operation_parser(token_iterator)?;
		lhs = Expression::BinaryOperation {
			operator: BinaryOperator::from_token_type(operator.type_).unwrap(),
			lhs:      Box::new(lhs),
			rhs:      Box::new(rhs),
		};
		maybe_operator =
			token_iterator.lookahead(1, &format!("Expected any of {:?}", operators)).map(|tokens| tokens[0]);
	}
	Ok(lhs)
}

fn parse_term(token_iterator: &mut TokenStream) -> Result<Expression, String> {
	parse_binary_operation(token_iterator, parse_factor, &[TokenType::Star, TokenType::Slash])
}

fn parse_factor(token_iterator: &mut TokenStream) -> Result<Expression, String> {
	let token = token_iterator.lookahead(1, "Expected expression")?[0];
	match token.type_ {
		TokenType::Plus | TokenType::Minus => {
			token_iterator.next("")?;
			let unary_operand = parse_factor(token_iterator)?;
			Ok(Expression::UnaryOperation(
				UnaryOperator::from_token_type(token.type_).unwrap(),
				Box::new(unary_operand),
			))
		},
		TokenType::IntegerLiteral => {
			let number_literal = token_iterator.next("")?.expect(TokenType::IntegerLiteral)?;
			// TODO: Use our own integer parser
			let value: i128 =
				number_literal.text().parse().map_err(|err| format!("Invalid number literal '{}'", err))?;
			Ok(Expression::NaturalLiteral(value))
		},
		_ => Err(format!("Unknown start of expression '{:?}'", token.type_)),
	}
}

#[cfg(test)]
mod test {
	extern crate test;
	use test::Bencher;

	use super::*;
	use crate::lexer;

	#[bench]
	fn bench_parser(bencher: &mut Bencher) {
		let file_contents = std::fs::read_to_string("../powder-dev/simple.pw").unwrap();
		let tokens = lexer::lex(&file_contents).unwrap();
		bencher.iter(move || parse(tokens.clone()));
	}

	fn evaluate_expression(expression: Expression) -> i128 {
		match expression {
			Expression::NaturalLiteral(value) => value,
			Expression::UnaryOperation(UnaryOperator::Plus, value) => evaluate_expression(*value),
			Expression::UnaryOperation(UnaryOperator::Minus, value) => -evaluate_expression(*value),
			Expression::BinaryOperation { lhs, operator: BinaryOperator::Add, rhs } =>
				evaluate_expression(*lhs) + evaluate_expression(*rhs),
			Expression::BinaryOperation { lhs, operator: BinaryOperator::Subtract, rhs } =>
				evaluate_expression(*lhs) - evaluate_expression(*rhs),
			Expression::BinaryOperation { lhs, operator: BinaryOperator::Multiply, rhs } =>
				evaluate_expression(*lhs) * evaluate_expression(*rhs),
			Expression::BinaryOperation { lhs, operator: BinaryOperator::Divide, rhs } =>
				evaluate_expression(*lhs) / evaluate_expression(*rhs),
			_ => panic!("Unsupported expression {:?} for evaluation", expression),
		}
	}

	static mut TOKEN_STREAMS: Vec<Vec<Token>> = Vec::<Vec<Token>>::new();

	fn create_tokens(code: &'static str) -> Result<TokenStream<'static>, String> {
		unsafe {
			TOKEN_STREAMS.push(lexer::lex(code)?);
			Ok(TokenStream::new(&TOKEN_STREAMS[TOKEN_STREAMS.len() - 1]))
		}
	}

	#[test]
	fn test_parse_expression() -> Result<(), String> {
		assert_eq!(evaluate_expression(parse_expression(&mut create_tokens("-15")?)?), -15);
		assert_eq!(evaluate_expression(parse_expression(&mut create_tokens("0 + 7 * 3 + 5")?)?), 26);
		assert_eq!(evaluate_expression(parse_expression(&mut create_tokens("1 - 5")?)?), -4);
		assert_eq!(evaluate_expression(parse_expression(&mut create_tokens("8 * 0 + 4")?)?), 4);
		assert_eq!(evaluate_expression(parse_expression(&mut create_tokens("22 - 6 *2+1")?)?), 11);
		assert_eq!(evaluate_expression(parse_expression(&mut create_tokens("2 - 2 - 1")?)?), -1);

		Ok(())
	}
}
