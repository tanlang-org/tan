use std::{error::Error, fmt};

use crate::parser::expr::Expr;

// tree-walk interpreter

// #TODO interpret or eval or execute?
// #TODO alternative names: Processor, Runner

pub struct Env {
    // #TODO
}

#[derive(Debug)]
pub enum EvalError {
    UnknownError,
}

impl Error for EvalError {}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            EvalError::UnknownError => "unknown error".to_string(),
        };
        write!(f, "eval error: {}", err)
    }
}

// #TODO accept AsRef<Expr>
pub fn eval(expr: &Expr, env: &mut Env) -> Result<Expr, EvalError> {
    let result = match expr {
        Expr::Do(list) => {
            let mut result = Ok(Expr::One);
            for expr in list {
                result = eval(expr.as_ref(), env)
            }
            result
        }
        Expr::List(list) => {
            // #TODO replace head/tail with first/rest
            // #TODO empty list should also be found in read/parse phase
            // #TODO could this arise in self-modifying code?
            let head = list.first().ok_or(EvalError::UnknownError)?;
            let tail = &list[1..];

            let Expr::Symbol(s) = head.as_ref() else {
                return Err(EvalError::UnknownError);
            };

            if s != "write" {
                return Err(EvalError::UnknownError);
            }

            let output = tail.iter().fold(String::new(), |mut str, x| {
                str.push_str(&format!("{}", x.as_ref()));
                str
            });

            println!("{output}");

            Ok(Expr::One)
        }
        _ => {
            // Unhandled expression variants evaluate to themselves.
            return Ok(expr.clone());
        }
    };

    result
}
