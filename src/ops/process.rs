use crate::{
    eval::{env::Env, error::EvalError},
    expr::Expr,
};

/// Terminates the current process with the specified exit code.
pub fn exit(args: &[Expr], _env: &Env) -> Result<Expr, EvalError> {
    if let Some(code) = args.first() {
        let Expr::Int(code) = code else {
            return Err(EvalError::ArgumentError("expected Int argument".to_owned()));
        };

        let code = *code as i32;

        std::process::exit(code);
    } else {
        // Exit with code=0 by default.
        std::process::exit(0);
    }
}

// #TODO args
// #TODO env