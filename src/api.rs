// #TODO find a better name, e.g. `lang`, `sys`, `runtime`.

use crate::{
    ann::Ann,
    error::Error,
    eval::{env::Env, eval},
    expr::Expr,
    lexer::Lexer,
    parser::Parser,
};

// #TODO all should return Ranged<Error> and Ann<Expr>.

/// Parses a Tan expression encoded as a text string.
pub fn parse_string(input: impl AsRef<str>) -> Result<Ann<Expr>, Error> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    Ok(expr)
}

/// Evaluates a Tan expression encoded as a text string.
pub fn eval_string(input: impl AsRef<str>, env: &mut Env) -> Result<Expr, Error> {
    let input = input.as_ref();

    let mut lexer = Lexer::new(input);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    let value = eval(expr, env)?;

    Ok(value)
}
