# The Powder Programming Language

Powder is a new programming language, that I (kleines Filmröllchen) am creating from-scratch. The primary goals with this project are:
* Explore creating a compiled systems language in the wake of [Rust](https://www.rust-lang.org), [Zig](https://ziglang.org), or [V](https://vlang.io).
* Explore compiling to [LLVM IR](https://llvm.org).
* Explore bootstrapping and self-compiling. The current implementation plan is to write a bootstrapping compiler in Rust, until I have enough features to be able to compile that state of the language within itself. At that point, I'll discard the bootstrapping compiler and continue developing Powder in itself, hopefully making me a very good Powder developer :^)
* Provide as complete as possible documentation in the form of [YouTube videos](https://www.youtube.com/c/kleinesfilmroellchen). I find that there's a distinct lack of video content on the language creation process not in a scripted, cleaned-up, and "just the good parts" form but in one that's more akin to [Andreas Kling's](https://www.youtube.com/c/AndreasKling) SerenityOS hacking. Unedited, direct content, including dead ends and struggles that someone uneducated like me will have.

The primary purpose of Powder is for me to have fun writing compilers. Also, I want to improve my Rust skills. So Powder has no intended purpose, especially when considering that Rust and Zig do many things better than it, even if I find that I'm improving on them in some places. Powder might never get "pressed" into a final form, so to speak.

## Design

Here is some rough language design. Note that while I'm still in early prototyping, this can change a lot. The intention is that the [test file](powder-dev/simple.pw) can always be parsed and/or compiled to some degree by the bootstrap compiler.

Powder is a strong statically typed, multi-paradigm (functional, structured, limited object oriented, generic) compiled programming language intended for systems programming. Powder intends to be minimal: It tries to be as powerful as possible with as little features as possible. Powder also intends to be orthagonal: If there's a rule, it's always followed. Everything behaves the same way, in syntax and semantics.

### Top-level constructs

The only two top-level constructs right now are global variables and functions. Global variables are normal variable declarations, see below. Functions follow this syntax:

```powder
(modifiers) function my_function(argument_1: Type, argument_2: OtherType, ...): ReturnType {
	// ... function body
}
```

Modifiers are either preprocessor attributes (from libraries or user definitions) or visibility modifiers. A function can have any number of arguments, including none, whose type needs to always be declared explicitly. The function's return type needs to be specified unless it is the unit type `()`, in which case it may be omitted. The function's body is a statement, it need not be a block as shown above.

### General constructs

There are some general language constructs used everywhere to compose other complex constructs:
* An expression is anything that may produce a result, which actually *is* anything in Powder. No exceptions.
* A statement is the same thing as an expression, though it is usually terminated with a semicolon `;` and its value is unused.
* A block is a kind of expression composed of a series of statements, enclosed in curly braces `{` `}`. The final expression (or statement; again they're the same thing) forms the return value of the block, though this is ignored if the block's return value is also ignored.

### Types

These are some of the planned built-in and user-defined types:
* Numeric types: integers (`i8`, `i16`, `i32`, `i64`), natural numbers ("unsigned integers", what a dumb term) (`n8`, `n16`, `n32`, `n64`), floating-point numbers (`f32`, `f64`), `bool`
* Text types: `char` (32-bit Unicode code point), `string` (validated UTF-8 sequence of arbitrary size)
* Primitive collection types: Array (statically-sized sequence), Slice (dynamically-sized sequence) `[T]`.
* Reference types: Pointers `*` & mutable pointers `*mut` (these are not lifetime-checked and are mainly intended for the bootstrap compiler), References `&` & mutable references `&mut`.
* User-defined structure types and (dynamically sized) interface types

### Variable declaration

A variable declaration has the form:
```powder
[const|variable] name: Type [= initializer_expression ];
```

For now, the type is mandatory. The initializer expression is optional. If no initial value is given, this can be done later as long as the variable is not used before it's assigned.

## License

[MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your discretion.

Copyright (c) 2022, kleines Filmröllchen
