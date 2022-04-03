use crate::lexer::TokenType;
use std::fmt::Debug;

pub trait Ast: Debug {}

#[derive(Debug)]
pub struct File {
	pub definitions: Vec<Definition>,
}
impl Ast for File {}

#[derive(Debug)]
pub enum Definition {
	Function(Function),
	// TODO: Only variable declarations are allowed.
	Statement(Statement),
}
impl Ast for Definition {}

#[derive(Debug)]
pub struct Function {
	pub name: String,
	pub body: Block,
}
impl Ast for Function {}

#[derive(Debug)]
pub struct Block {
	pub statements: Vec<Statement>,
	pub value: Option<Expression>,
}
impl Ast for Block {}

#[derive(Debug)]
pub enum Statement {
	Expression(Expression),
	VariableDeclaration {
		kind: VariableKind,
		name: String,
		type_: Type,
		initial_value: Option<Expression>,
	},
}
impl Ast for Statement {}

#[derive(Debug)]
pub enum Expression {
	NaturalLiteral(i128),
	UnaryOperation(UnaryOperator, Box<Expression>),
	BinaryOperation {
		operator: BinaryOperator,
		lhs: Box<Expression>,
		rhs: Box<Expression>,
	},
}
impl Ast for Expression {}

#[derive(Debug)]
pub enum VariableKind {
	/// `const`
	Immutable,
	/// `var`
	Mutable,
}

#[derive(Debug)]
pub enum UnaryOperator {
	Plus,
	Minus,
	Not,
	Reference,
	Dereference,
}

impl UnaryOperator {
	pub const fn from_token_type(token_type: TokenType) -> Option<Self> {
		match token_type {
			TokenType::Plus => Some(Self::Plus),
			TokenType::Minus => Some(Self::Minus),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub enum BinaryOperator {
	Add,
	Subtract,
	Multiply,
	Divide,
}

impl BinaryOperator {
	pub const fn from_token_type(token_type: TokenType) -> Option<Self> {
		match token_type {
			TokenType::Plus => Some(Self::Add),
			TokenType::Minus => Some(Self::Subtract),
			TokenType::Star => Some(Self::Multiply),
			// TODO: Parse / individually.
			_ => None,
		}
	}
}

#[derive(Debug)]
pub enum Type {
	N64,
}
