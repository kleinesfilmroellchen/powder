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
	NaturalLiteral(u128),
	UnaryOperation(UnaryOperator, Box<Expression>),
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

#[derive(Debug)]
pub enum Type {
	N64,
}
