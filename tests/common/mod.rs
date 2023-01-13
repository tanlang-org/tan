//! Common testing-support functions and utilities.

use tan::{
    ann::Ann,
    eval::{env::Env, error::EvalError, eval},
    expr::Expr,
    lexer::{error::LexicalError, token::Token, Lexer},
    parser::{error::ParseError, Parser},
    range::Ranged,
};

// #TODO reuse api.rs?

pub fn lex_string(input: &str) -> Result<Vec<Ranged<Token>>, Ranged<LexicalError>> {
    let mut lexer = Lexer::new(input);
    lexer.lex()
}

pub fn parse_string(input: &str) -> Result<Ann<Expr>, Ranged<ParseError>> {
    // #TODO surface LexicalError!
    let tokens = lex_string(input).unwrap();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

pub fn eval_string(input: &str) -> Result<Expr, EvalError> {
    // #TODO surface ParseError!
    let expr = parse_string(input).unwrap();
    let mut env = Env::prelude();
    eval(&expr, &mut env)
}

pub fn read_file(filename: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{filename}")).unwrap()
}

#[allow(dead_code)]
pub fn lex_file(filename: &str) -> Result<Vec<Ranged<Token>>, Ranged<LexicalError>> {
    let input = &read_file(filename);
    lex_string(input)
}

#[allow(dead_code)]
pub fn parse_file(filename: &str) -> Result<Ann<Expr>, Ranged<ParseError>> {
    let input = &read_file(filename);
    parse_string(input)
}

#[allow(dead_code)]
pub fn eval_file(filename: &str) -> Result<Expr, EvalError> {
    let input = &read_file(filename);
    eval_string(input)
}
